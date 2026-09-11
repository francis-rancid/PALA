# Chapter 19  
 Linux Memory Acquisition

This chapter provides the fundamental knowledge you need to begin analyzing Linux memory dumps. In particular, we discuss historical and modern memory acquisition techniques on Linux, as well as the advantages and disadvantages of each approach. You will learn how to create Linux profiles, which are archives that contain the necessary information Volatility needs to properly find and interpret data in Linux memory dumps. Additionally, we discuss the challenges of deploying Linux memory forensics in an enterprise environment, where critical servers may not even have C compilers or other libraries that are found on standard Linux desktops and workstations.

---

**NOTE**

Unless otherwise noted, the best practices you learned regarding safe memory acquisition procedures on Windows in Chapter 4 also apply to Linux systems.

---

## Historical Methods of Acquisition

Initial methods of memory acquisition on Linux did not require third-party software. Instead, interfaces built into the operating system allowed for reading and writing of physical memory by privileged applications. For example, you could read `/dev/mem` (described in the next section) with `cat` or `dd` and redirect it to a file or over the network. Due to the security hazard posed by such interfaces, they are now disabled or crippled in order to prevent abuse. As a side effect, disabling or crippling the interfaces also prevents forensics investigators from using them as facilitators of memory acquisition.

### /dev/mem

Before being disabled on nearly all distributions due to security concerns, the most popular interface for memory acquisition was `/dev/mem`. When enabled, this device exports physical memory and allows programs (such as `dd`) with root privileges to directly read from and write to RAM. Unfortunately, `/dev/mem` also presented challenges for inexperienced investigators. First, many machines do not map RAM contiguously from physical offset 0. As a result, an investigator might access sensitive regions, causing memory corruption or general system instability, as described in Chapter 4.

Another issue with `/dev/mem` is that it can only address the first 896MB of RAM, even if the system’s physical memory capacity is far greater. Although early forensic investigations frequently involved machines with less than 896MB of RAM, it is almost unheard of now, and the inability to acquire memory beyond that range is a limitation associated with using `/dev/mem`.

### /dev/kmem

The `/dev/kmem` character device was historically used to acquire a subset of memory on 32-bit systems before being disabled. Whereas `/dev/mem` exports raw physical memory, /dev/kmem exports the kernel virtual address space. Similar to `/dev/mem`, the security risk associated with allowing userland direct access to kernel memory led to the decision to disable `/dev/kmem` by default on modern distributions.

### ptrace

`ptrace` is the userland debugging interface that Linux provides. The interface is not suitable for robust memory acquisition, because it can acquire pages only from running processes, which misses all kernel memory, freed pages, and other data. The only time you should use `ptrace` as an acquisition method is when you are interested in the code and data of only one process, such as related to a piece of malware.

If you want to experiment with a `ptrace`-based memory acquisition tool, you can try the `memfetch` application available here: [http://lcamtuf.coredump.cx/soft/memfetch.tgz](http://lcamtuf.coredump.cx/soft/memfetch.tgz). This application runs natively on Intel Linux and is trivially portable to ARM. It operates by reading the starting and ending addresses of process’ memory ranges from `/proc/<pid>/maps` and then uses `ptrace` to dump each page to disk. Later in the book, you will learn how the kernel populates `/proc/<pid>/maps` and how to use the `linux_dump_map` Volatility plugin to achieve similar effects from a full memory sample.

## Modern Acquisition

The inadequacies of the previously described historical methods of acquisition on 32-bit systems caused developers in the forensics community to create specialized memory acquisition tools. The tools provide practitioners more flexibility over the acquisition process, but also require that you load third-party software onto target systems in order to collect the evidence. With the advent of 64-bit systems, another userland device, `/proc/kcore`, once again opened the possibility of acquisition of physical memory directly from userland. In this section we discuss `/proc/kcore`, the `fmem` and `LiME` acquisition tools, and the advantages and disadvantages of each.

### fmem

To alleviate issues encountered when using `/dev/mem` on 32-bit systems, the `fmem` ([http://hysteria.sk/~niekt0/foriana/fmem_current.tgz](http://hysteria.sk/~niekt0/foriana/fmem_current.tgz)) project was created. `fmem` operates by loading a kernel driver that creates a character device named `/dev/fmem`. This device operates similarly to `/dev/mem`, in that it exports physical memory for other programs to access, but it has a number of advantages. The first advantage is that it checks if physical pages reside in main memory (by calling the `page_is_ram` function) before accessing them. This prevents investigators from accidentally reading device memory or unmapped physical addresses, which causes stability issues (see Chapter 4). Another advantage of `fmem` is that it can access physical pages above the 896MB boundary, unlike `/dev/mem`.

The only disadvantage to `fmem` is that it requires the investigator to inspect `/proc/iomem` to determine where RAM is mapped. As previously discussed, this is necessary for machines that do not map all main memory at physical offset 0. On some systems, RAM is broken into many different segments that are loaded high into physical memory. For example, the following is the initial output of `/proc/iomem` on a VMware workstation Debian guest with 3084MB of physical memory and PAE enabled:

```
$ cat /proc/iomem
00000000-0000ffff : reserved
00010000-0009f3ff : System RAM
0009f400-0009ffff : reserved
000a0000-000bffff : PCI Bus 0000:00
  000a0000-000bffff : Video RAM area
000c0000-000c7fff : Video ROM
000ca000-000cbfff : reserved
  000ca000-000cafff : Adapter ROM
000cc000-000cffff : PCI Bus 0000:00
000d0000-000d3fff : PCI Bus 0000:00
[snip]
```

In this output, you can see the starting and ending physical address of each memory range along with the name. The ones you should be interested in for acquisition are those named `System RAM`. To find these ranges exclusively, you can search for them in the file:

```
$ grep "System RAM" /proc/iomem
00010000-0009f3ff : System RAM
00100000-bfedffff : System RAM
bff00000-bfffffff : System RAM
100000000-100bfffff : System RAM
```

Within this guest, there are four ranges of physical memory that you need to acquire. Note that the last range (`100000000` - `100bfffff`) is above the normal limit for 32-bit systems (which is `0xffffffff`) and actually is at about 4.3GB into physical memory. So if you use `dd` to acquire 3084MB of RAM with `fmem`, you would miss data in the last region. It would take a knowledgeable investigator to notice this error because the file would be the correct size of RAM (3084MB), and memory analysis tools (including Volatility) would appear to work for the most part. However, the memory sample would be incomplete.

Although `fmem` is a great addition to the memory acquisition and research space, it is a very manual tool and it requires you to have intimate knowledge of the physical memory layout to use it properly.

### Linux Memory Extractor (LiME)

Linux Memory Extractor, otherwise known as LiME ([https://code.google.com/p/lime-forensics](https://code.google.com/p/lime-forensics)), is the latest Linux memory acquisition tool. It fixes the issues and challenges involved with the previously discussed tools and techniques. Joe Sylve, Vico Marziale, Andrew Case, and Golden G. Richard III discuss the original implementation of LiME in a paper that you can read here: [http://www.dfir.org/research/android-memory-analysis-DI.pdf](http://www.dfir.org/research/android-memory-analysis-DI.pdf).

LiME operates by loading a kernel driver, but instead of creating a character device that userland can access, all acquisition is done within the kernel. This significantly enhances the accuracy of the resulting sample because there are no context switches between userland and the kernel required to transfer data. LiME also improves upon `fmem` by automatically determining the address ranges that contain main memory. To enumerate the ranges, LiME walks the kernel’s `iomem_resource` linked list. This list holds the descriptors for each physical memory segment. If the name of the segment matches that of RAM (`System RAM`), the corresponding `start` and `end` members determine where in physical memory the segment resides.

When acquiring memory with LiME, you can choose between multiple acquisition formats. When the recommended `lime` format is used, a structured file is produced that removes the need to create a zero-padded file. This structured format contains metadata that describes from which physical offset each section came, which allows the memory analysis tool to dynamically reconstruct the original memory layout.

Each segment of acquired memory starts with a `lime_header`. The following output shows this metadata structure:

```
>>> dt("lime_header")
'lime_header' (32 bytes)
0x0   : magic                          ['unsigned int']
0x4   : version                        ['unsigned int']
0x8   : start                          ['unsigned long long']
0x10  : end                            ['unsigned long long']
0x18  : reversed                       [‘unsigned long long']
```

The header contains a `magic` member whose value is `0x4C694D45` (LiME in hexadecimal). The `start` and `end` members tell you which addresses in physical memory correspond to the segment. The actual data immediately follows `lime_header`. To find the next segment and its `lime_header`, you simply add the size of the header plus the size of the segment. In Volatility, this logic is implemented within the LiME address space in `volatility/plugins/addrspaces/lime.py`.

#### Compiling LiME

To use LiME, you must first compile the kernel module for the kernel version you want to analyze. This is covered in detail in the project’s documentation ([https://code.google.com/p/lime-forensics/downloads/list](https://code.google.com/p/lime-forensics/downloads/list)). After you have compiled the module, you will have a file named `lime-<kernel version>.ko` that you can load on the target system (see the next step).

---

**WARNING**

Unless absolutely necessary, you should never compile LiME (or any software, for that matter) on a system that you plan to investigate. Compilation requires the installation of source code and other supporting files, not to mention that it creates a number of temporary files during the build process that might overwrite evidence in slack space. In some instances, the target machine may not even have a compiler installed. Thus, we strongly recommend compiling your tools on a system that is not part of the investigation and then executing only the compiled file(s) on the target machine.

Ideally, you would have access to test systems running the same kernel versions as your production ones. Leverage those machines to compile the LiME module. If you don’t have similarly configured systems, you can cross-compile modules (for more information, see the “Enterprise Linux Memory Forensics” section later in the chapter).

---

#### Loading LiME and Dumping Memory

LiME has the capability to dump memory to either the local disk or over the network, and it can do so in the following formats:

- **raw**: All memory ranges concatenated together
- **padded**: Similar to the raw format, except that gaps between memory ranges are padded with zeroes
- **lime**: Writes to the aforementioned LiME format (recommended)

You can control the destination of the memory dump by setting the `path` parameter to the kernel module. To dump the contents of memory to a file on a storage device, just specify `path=/path/to/memdmp.lime`. In this case, memory is acquired and written to your desired location as soon as the module is loaded. An example of acquiring memory to disk in the `lime` format is shown in the following command:

```
$ sudo insmod lime.ko "path=/mnt/externaldrive/memdmp.lime format=lime"
```

Note that in this command, we specify the path as `/mnt/external` in order to dump memory to an external drive. You should never, unless absolutely necessary, save evidence to the local disk because it would overwrite unallocated disk space that may be helpful during disk forensics.

For network-based acquisition, the path is given as `path=tcp:4444` (the actual port is up to you). A listening socket is then created on the target system and specified port. You can connect to the socket with a tool such as `netcat`, and acquisition will begin as soon as the connection is established. An example of acquiring memory over the network with `netcat` follows. The first command is run on the machine from which you want to acquire memory:

```
$ sudo insmod lime.ko "path=tcp:4444 format=lime"
```

Then on your forensics station, you can use `netcat` to acquire (assuming that 192.168.1.40 is the IP address of the target computer in which LiME is installed):

```
$ nc 192.168.1.40 4444 > memdmp.lime
```

---

**NOTE**

On some distributions, such as Ubuntu, the quotes around the parameters are required for correct operation. Other distributions, such as Red Hat, do not operate correctly if quotes are used.

---

### /proc/kcore

The `/proc/kcore` file exports the kernel’s virtual address space to userland in the form of a core dump (ELF) file. On 32-bit systems, this limits acquisition to the first 896MB of memory as explained in the `/dev/mem` discussion. On 64-bit systems, the usability of `/proc/kcore` changes as the kernel keeps a static virtual map of all RAM. This mapping is documented by the kernel developers in the `Documentation/x86/x86_64/mm.txt` file of the kernel source code. By reading from this static map, you can successfully acquire physical memory from 64-bit systems through `/proc/kcore`.

Unfortunately, the steps to acquire memory from `/proc/kcore` are tedious, and often require a specialized tool. A proof-of-concept- tool that can acquire memory from `/proc/kcore` accompanies the Volatility 2.4 release. The tool works by parsing the ELF sections of `/proc/kcore` and determining the segments that map memory through comparison with data read from `/proc/iomem`. Each acquired region is then written as a `LiME` segment in order to be immediately compatible with Volatility.

Even on 64-bit systems, `/proc/kcore` has a few disadvantages that may necessitate the use of the actual `LiME` tool for acquisition. The first issue is that `/proc/kcore` can be disabled. Nearly all stock distributions enable `/proc/kcore`, but security conscience systems such as grsecurity and the hardened Gentoo distribution disable it. The other disadvantage is that malware can hook the read function of `/proc/kcore` and filter out data received by the userland tool.

## Volatility Linux Profiles

Before you can use Volatility to analyze your newly acquired memory sample, you must first create a profile for the target operating system. If you have performed analyses on Windows samples before, you know that Volatility has built-in support for all the major Windows versions, and that no extra steps are required. However, Linux is different due to the large number of kernel versions, subkernel versions, and customized kernels. For example, at the time of this writing, Volatility supports kernel versions 2.6.11 through 3.14 (the latest). There are over 40 base kernels and 500 sub-versions in this range, each requiring a separate profile. Additionally, when a user compiles a kernel from scratch, each configuration option (enabled or disabled) can drastically change the resulting kernel.

This large number of kernel versions makes it impossible for Volatility to ship with profiles for all possible builds of the Linux kernel. Instead, the current approach is to build profiles for the most commonly seen kernels and educate users on how to create their own profiles for their own systems. In the future, we hope to have a more automated system for building a wide range of profiles that users can download.

---

**NOTE**

Hal Pomeranz released a tool (`https://github.com/halpomeranz/lmg`) that automates the compilation of both Volatility profiles and LiME modules when you execute from a system running the target kernel. This tool can greatly speed up analysis and ease the learning curve for new investigators.

---

### Software Setup

The software required to create Linux profiles is identified in the following listing:

- **dwarfdump**: A tool that parses the debugging information from ELF files, such as the Linux kernel and kernel modules. Specifically, the `dwarfdump` output includes the structure definitions. You can install this tool by fetching the `dwarfdump` package on Ubuntu/Debian, using the `libdwarf-tools` package on OpenSuSE and Fedora, or compiling it from source code ([http://reality.sgiweb.org/davea/dwarf.html](http://reality.sgiweb.org/davea/dwarf.html)).
- **Compiler tools**: You must install the tools required to compile C source code, such as `gcc` and `make`, on your computer. On Ubuntu and Debian distributions, you can simply use `apt-get install build-essential`, but this will vary between distributions.
- **Kernel headers**: You must retrieve the kernel headers of the exact kernel you want to analyze in order to create the correct structure definitions. Ubuntu/Debian headers are packaged in the `linux-headers-`uname -r`` package. For other distributions, simply search for their notion of kernel headers within the repository.

### Creating a Profile

Creating a profile consists of generating a set of VTypes (structure definitions) and a `System.map` file for a particular kernel version. Volatility leverages these sources of information to perform its analysis.

#### Creating VTypes

The current method to create VTypes is to compile `tools/linux/module.c` (distributed with Volatility) against the kernel that you want to analyze. `module.c` is a kernel module that declares members of all the types that Volatility needs. These declarations are enough to add the type definitions into the module’s debugging information. To compile the module, simply change into the `tools/linux` directory and type `make`. If successful, your output should look similar to this:

```
$ make
make -C //lib/modules/3.2.0-4-686-pae/build 
  CONFIG_DEBUG_INFO=y M=/opt/vol2.4/tools/linux modules
make[1]: Entering directory `/usr/src/linux-headers-3.2.0-4-686-pae'
  CC [M]  /opt/vol2.4/tools/linux/module.o
  Building modules, stage 2.
  MODPOST 1 modules
  CC      /opt/vol2.4/tools/linux/module.mod.o
  LD [M]  /opt/vol2.4/tools/linux/module.ko
make[1]: Leaving directory `/usr/src/linux-headers-3.2.0-4-686-pae'
dwarfdump -di module.ko > module.dwarf
make -C //lib/modules/3.2.0-4-686-pae/build M=/opt/vol2.4/tools/linux clean
make[1]: Entering directory `/usr/src/linux-headers-3.2.0-4-686-pae'
  CLEAN  /opt/vol2.4/tools/linux/.tmp_versions
  CLEAN  /opt/vol2.4/tools/linux/Module.symvers
make[1]: Leaving directory `/usr/src/linux-headers-3.2.0-4-686-pae'
```

Note that near the end of the compilation, `dwarfdump` runs against `module.ko` (the kernel module) to produce `module.dwarf`. You include this file within the profile zip.

---

**NOTE**

Building the `module.ko` is not necessary if you can use the compiled Linux kernel file (`vmlinux`) instead. However, the `vmlinux` file is not supplied with most Linux distributions.

---

#### Getting Symbols

Symbols are contained within the `System.map` file. You find this file in a number of places, including the install package for your distribution’s kernel, the `/boot` directory of the machine where the kernel is installed, or the source code directory in which the kernel was compiled. If you have the kernel ELF file (`vmlinux`), you can also generate a `System.map` file by running the `nm` command against it.

Most people simply copy the file from `/boot`, but be careful because systems with multiple kernels have multiple `System.map` files. In such cases, you can verify the filename first with `uname -a`, or on most distributions you can copy `/boot/System.map-`uname -r`` in order to ensure that you select the correct file. The `r` flag to `uname` tells it to print only the kernel release.

---

**NOTE**

The `System.map` file contains the addresses of all symbols from the kernel, which Volatility uses to locate key data structures in memory. Even a simple recompile of the same kernel is enough to change the addresses of symbols. These modifications are a major reason why you cannot take a profile you built for Ubuntu 11.04 and expect it to work on Ubuntu 12.04. You also cannot expect to run `apt-get upgrade` and have your previously functioning profile work with the newly updated kernel.

---

#### Making the Profile

To create the profile, place the `module.dwarf` and `System.map` files into a zip file. Then move this zip file into the `volatility/plugins/overlays/linux/` directory. You should name the zip file according to the distribution, architecture, and kernel version that it represents. You can take care of all these steps with a single command, as shown here:

```
$ zip /path/to/volatility/plugins/overlays/linux/Redhat2.6.11.zip 
      /path/to/module.dwarf 
      /path/to/System.map
```

---

**NOTE**

You can also create a separate directory outside of Volatility to store your profiles. When running Volatility, you must then prepend your command-line arguments with `--plugins=/path/to/your/profile/directory`. This can be useful if you are using the standalone Windows executable to analyze Linux memory dumps—in which case you don’t have a source installation of Volatility.

---

### Using the Profile

To use the profile, you must first find the name that Volatility assigns to it. The name will be “Linux” + your zip file name + “x86,” “x64,” or “ARM,” depending on the architecture. However, in case you forget, you can look up the name by running the `--info` command and searching for “Linux”:

```
$ python vol.py --info | grep Linux
Volatility Foundation Volatility Framework 2.4
LinuxCentOS63x64                 - A Profile for Linux CentOS63 x64
LinuxCentOS53x64                 - A Profile for Linux CentOS5.3 x64
LinuxFedora17x64                 - A Profile for Linux Fedora17 x64
LinuxNovellSuSE111x64            - A Profile for Linux NovellSuSE111 x64
LinuxOpenSuSE12x86               - A Profile for Linux OpenSuSE12 x86
LinuxUbuntu1204x64               - A Profile for Linux Ubuntu1204 x64
```

In this example, you see that a number of Linux profiles are installed, and this is a good example of why a consistent naming convention of your zip files is important. To use the given profile, simply pass its name as the `--profile` parameter:

```
$ python vol.py --profile=LinuxFedora17x64 -f /path/to/memory/sample linux_pslist
```

### Enterprise Linux Memory Forensics

Although creating profiles for one or two operating system versions is not overly difficult or burdensome, Linux systems administrators are often tasked with supporting multiple operating systems and kernel versions across the enterprise. Similarly, incident response teams often walk into environments with little or no preexisting knowledge of the operating system(s) in use.

To help in these situations, Volatility contains a separate `Makefile` (`Makefile.enterprise`) that allows you to cross compile (that is, compile against an arbitrary set of kernel headers instead of those of the current system). To use this `Makefile`, you must first edit the second line (`KDIR`) to point to your kernel headers directory. Then, instead of typing `make` from the `tools/linux` directory, type `make -f Makefile.enterprise`. This command compiles the kernel module against your custom headers.

#### Caveats to Cross Compiling

One complication you may encounter during the cross compilation process is that the kernel headers package from the distribution does not have the `.config` file needed to compile the Volatility debug module (`module.c`). In this case, you must also download the kernel image package. You then copy the file named `config-<kernel version>` to `.config` under the kernel headers directory. You also need to retrieve the `System.map` file for the kernel you want to compile against. This file is contained in the same kernel image package where the `.config` file is located.

#### Cross Compiling for Ubuntu

To compile a profile for Ubuntu’s 2.6.32-22-generic x86 kernel, you need the `linux-image-2.6.32-22-generic`, `linux-headers-2.6.32-22`, and `linux-headers-2.6.32-22-generic` packages. They are found on Ubuntu’s package search website: [http://packages.ubuntu.com](http://packages.ubuntu.com). Once you have the `linux-image` package, extract the `.deb` file with `ar` and then extract the data from the tar archive, as shown in the following commands:

```
$ wget http://security.ubuntu.com/ubuntu/pool/main/l/\
linux/linux-image-2.6.32-22-generic_2.6.32-22.36_i386.deb
[snip]
2013-11-23 11:54:51 - `linux-image-2.6.32-22-generic_2.6.32-22.36_i386.deb' 
$ ar x linux-image-2.6.32-22-generic_2.6.32-22.36_i386.deb
$ tar -xjf data.tar.bz2
$ ls
boot  data.tar.bz2  lib  usr
$ ls boot
abi-2.6.32-22-generic  config-2.6.32-22-generic  System.map-2.6.32-22-generic  
vmcoreinfo-2.6.32-22-generic  vmlinuz-2.6.32-22-generic
```

As you can see in the extracted data files, you have directories named `boot`, `lib`, and `usr`. If you list the contents of the `boot` directory, you find the `System.map` and `.config` file for the particular kernel version. In this example, the `.config` file is named `config-2.6.32-22-generic.`

Next, you have to download two packages for the kernel headers because Ubuntu makes a generic package for each kernel version plus specific packages for each version used by the operating system. You can start by downloading the base package:

```
$ wget http://security.ubuntu.com/ubuntu/pool/main/\
l/linux/linux-headers-2.6.32-22_2.6.32-22.36_all.deb
[snip]
2013-11-23 12:27:28 - `linux-headers-2.6.32-22_2.6.32-22.36_all.deb' 
$ ar x linux-headers-2.6.32-22_2.6.32-22.36_all.deb
$ tar -xzf data.tar.gz
$ ls usr/src/linux-headers-2.6.32-22
arch   crypto         drivers   fs
[snip]
```

Next you download the package for your specific kernel version:

```
$ wget http://security.ubuntu.com/ubuntu/pool/main/\
l/linux/linux-headers-2.6.32-22-generic_2.6.32-22.36_i386.deb
[snip]
2013-10-23 11:53:38 - `linux-headers-2.6.32-22-generic_2.6.32-22.36_i386.deb' 
$ ar x linux-headers-2.6.32-22-generic_2.6.32-22.36_i386.deb
$ ls
control.tar.gz  data.tar.gz  debian-binary  
linux-headers-2.6.32-22-generic_2.6.32-22.36_i386.deb
$ tar -xzf data.tar.gz
$ ls
lib  usr
$ ls usr/src/linux-headers-2.6.32-22-generic/
arch   crypto drivers   fs
[snip]
```

At this point, you must create a directory and put both of the header folders under it:

```
$ mkdir headers
$ cd headers
$ mv /path/to/usr/src/linux-headers-2.6.32-22-generic/ .
$ mv /path/to/usr/src/linux-headers-2.6.32-22 .
$ ls
linux-headers-2.6.32-22  linux-headers-2.6.32-22-generic
```

Next, copy the `.config` file from the `linux-image` package under `linux-headers-2.6.32-22-generic` (the directory for your specific kernel) to the directory that was just created (headers). Finally, update your Makefile’s `KDIR` option to specify this directory. After running the `make` command, you will have the `module.dwarf` file that you can place into a zip file with the `System.map` file from the `linux-image` package. Now you have an Ubuntu profile that you created without having to install or use a computer running the specific kernel version.

## Summary

Historically, the Linux operating system has provided several interfaces into physical memory *and* the kernel virtual address space. However, none of the options are suitable for forensic acquisition of main memory in a safe and complete manner. LiME, on the other hand, is specifically designed to address the issues that other tools presented in the past. With the ability to cross compile, you can create and deploy LiME modules throughout heterogeneous enterprise environments. Using similar tactics, you can also build the appropriate Volatility profiles for the target systems.
