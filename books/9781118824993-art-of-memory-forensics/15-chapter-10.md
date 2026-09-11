# Chapter 10  
 Registry in Memory

The registry contains various settings and configurations for the Windows operating system, applications, and users on a computer. As a core component of a Windows machine, it is accessed constantly during run time. Thus, it makes sense that the system caches all or part of the registry files in memory. Furthermore, the Windows registry holds a wealth of information useful for forensic purposes. For example, you can use it to determine what programs recently ran, extract password hashes for auditing purposes, or investigate keys and values that malicious code introduced into the system.

In this chapter, you learn how to find and access registry files in memory by walking through examples of some of the aforementioned scenarios. Furthermore, you’ll be exposed to the difference between stable and volatile registry data, and how examining hives in memory can open up a whole new realm of analysis that isn’t possible with disk forensics.

## Windows Registry Analysis

The initial research on accessing registry files in memory was done by Brendan Dolan-Gavitt in 2008. His paper *Forensic Analysis of the Windows Registry in Memory* ([dfrws.org/2008/proceedings/p26-dolan-gavitt.pdf](http://dfrws.org/2008/proceedings/p26-dolan-gavitt.pdf)) and his original code provided the pioneering research upon which all of Volatility’s current registry capabilities are built.

---

## **Analysis Objectives**

Your objectives are these:

- **Locate registry files:** Understand how Volatility locates registry files consistently throughout memory dumps of various operating system versions.
- **Parse registry data:** Learn how to translate addresses to find keys, and how to print their subkeys, values, and data.
- **Discover forensically relevant registry keys:** Find and extract forensically relevant registry keys, such as those malware uses to maintain persistence.
- **Learn about special registry keys:** Userassist, Shimcache, and Shellbags are all registry keys that contain binary data that requires extra processing. You will learn the structures for these keys as well as when and how to use them in an investigation.

## **Data Structures**

The following output shows select structures from 32-bit Windows 7. The `_CMHIVE` structure represents a registry hive file on disk, and the `_HHIVE` (hive header) helps describe the contents and current state of the hive.

```
>>> dt("_CMHIVE")
'_CMHIVE' (1584 bytes)
0x0   : Hive                           ['_HHIVE']
0x2ec : FileHandles                    ['array', 6, ['pointer', ['void']]]
0x304 : NotifyList                     ['_LIST_ENTRY']
0x30c : HiveList                       ['_LIST_ENTRY']
0x314 : PreloadedHiveList              ['_LIST_ENTRY']
0x31c : HiveRundown                    ['_EX_RUNDOWN_REF']
0x320 : ParseCacheEntries              ['_LIST_ENTRY']
0x328 : KcbCacheTable                  ['pointer', ['_CM_KEY_HASH_TABLE_ENTRY']]
[snip]
0x3a0 : FileFullPath                   ['_UNICODE_STRING']
0x3a8 : FileUserName                   ['_UNICODE_STRING']
0x3b0 : HiveRootPath                   ['_UNICODE_STRING']
[snip]

>>> dt("_HHIVE")
'_HHIVE' (748 bytes)
0x0   : Signature                      ['unsigned long']
0x4   : GetCellRoutine                 ['pointer', ['void']]
0x8   : ReleaseCellRoutine             ['pointer', ['void']]
0xc   : Allocate                       ['pointer', ['void']]
0x10  : Free                           ['pointer', ['void']]
0x14  : FileSetSize                    ['pointer', ['void']]
0x18  : FileWrite                      ['pointer', ['void']]
0x1c  : FileRead                       ['pointer', ['void']]
0x20  : FileFlush                      ['pointer', ['void']]
0x24  : HiveLoadFailure                ['pointer', ['void']]
0x28  : BaseBlock                      ['pointer', ['_HBASE_BLOCK']]
[snip]
0x58  : Storage                        ['array', 2, ['_DUAL']]
```

## **Key Points**

The key points are these:

- `Hive`: The registry hive header that contains a signature as well as a structure used for address translation.
- `HiveList`: A doubly linked list to other `_CMHIVE` structures.
- `FileFullPath`: The kernel device path (for example, `\Device\HarddiskVolume1\WINDOWS\system32\config\software`) to the registry hive. This member is not used in Windows 7, although it is still present in the structure (see [http://gleeda.blogspot.com/2011/04/windows-registry-paths.html](http://gleeda.blogspot.com/2011/04/windows-registry-paths.html)).
- `FileUserName`: The path to the registry hive on disk, prefaced with `SystemRoot` or `\??\C:\` (except the `BCD` registry, which uses the kernel device path, and the `HARDWARE` registry, which uses the “Registry” path as described for the `HiveRootPath` member). Some hives do not use this member in Windows Vista/2008 and 7.
- `HiveRootPath`: Introduced in Windows Vista, this member contains the “Registry” path (for example, `\REGISTRY\MACHINE\SOFTWARE`).
- `Signature`: The signature of the registry file. Valid registry files have a signature of `0xbee0bee0`.
- `BaseBlock`: Used to find the root key (first key) of the registry.
- `Storage`: The mapping of the virtual address spaces for keys within the registry.

---

### Data in the Registry

To see how much information could be obtained from the registry in memory on a running system, Brendan Dolan-Gavitt conducted experiments on 32-bit XP machines in various states. He concluded that 98% of hive data is recoverable on lightly used systems, and about 50% of hive data is recoverable on heavily used systems. As with most artifacts in memory, the “use-it-or-lose-it” strategy is in effect. Thus, keys and data not accessed frequently (or recently) can be, and often are, swapped to disk. This is important to keep in mind when you analyze the registry in memory dumps. For example, the presence of a key in memory is evidence that the key existed on the machine at the time of acquisition. However, the absence of a key does not necessarily mean that the key didn’t exist—it could just be missing due to paging or it might not have been read into memory in the first place.

From a forensic standpoint, you can find a plethora of information in the registry. The following list summarizes a few of the possibilities:

- **Auto-start programs:** Identify applications that run automatically when the system starts up or a user logs in.
- **Hardware:** Enumerate the external media devices that were connected to the system.
- **User account information:** Audit user passwords, accounts, most recently used (MRU) items, and user preferences.
- **Recently run programs:** Determine what applications executed recently (using data from the Userassist, Shimcache, and MUICache keys).
- **System information:** Determine system settings, installed software, and security patches that have been applied.
- **Malware configurations:** Extract data related to malware command and control sites, paths to infected files on disk, and encryption keys (anything malicious code writes to the registry).

### Stable and Volatile Data

In addition to the aforementioned items commonly used in disk forensics, some volatile registry keys and hives are found only in memory. Jamie Levy did research for her talk, *Time is on My Side* ([http://gleeda.blogspot.com/2011/08/volatility-20-and-omfw.html](http://gleeda.blogspot.com/2011/08/volatility-20-and-omfw.html)), which showed that quite a bit of information is stored only in memory. For example, you can find data on volumes, devices, and settings. In just the `SYSTEM` hive and one user’s `NTUSER.DAT` hive, we counted more than 400 volatile keys.

A close relationship exists between the stable keys found in the registry on disk and those in memory. As a machine runs, new keys are created, and others change. It makes sense that these modifications are saved back to disk at some point. It was shown by Russinovich in *Microsoft Windows Internals, 6th Edition*, that data is flushed back to the disk every five seconds if Windows APIs (for example, `RegCreateKeyEx`, `RegSetValueEx`) are used. Brendan showed in his paper that if the registry is manipulated directly in memory without using the Windows APIs, however, the changes do not get flushed back to disk at all. During his research, he demonstrated this by performing the following steps:

1. Find the administrator password hash in memory.
2. Modify memory directly to change the value to a password hash for a known password.
3. Log out of the system (so the LSA subsystem would notice the change and update).
4. Log back in with the new password.

Because the changes would never get flushed back to disk, you would not know that this type of attack had occurred by just performing disk forensics. However, with a memory sample, this type of attack is simple to detect by dumping the password hashes from the registry hive and comparing them with the ones on disk.

### Finding Registry Hives

Volatility finds registry hives in memory by using the pool scanning approach (see Chapter 5). The `_CMHIVE` structure is allocated in a pool with the tag `CM10`. After you find such an allocation, you can verify that there is a valid hive by examining the `Signature` member (`_CMHIVE.Hive.Signature`). At this point, you can use the `HiveList` member to locate all the other hives (`_CMHIVE.HiveList`). The `_CMHIVE` structure is shown in [Figure 10-1](#figure10-1).

![c10f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c10/c10f001.png)

*[**Figure 10-1:**](#figureanchor10-1) Registry hives are enumerated by pool tag scanning and walking the linked list*

The `hivelist` plugin scans for registry hives and then prints out their physical and virtual offsets and path information. The following is an example:

```
$ python vol.py -f win7.vmem --profile=Win7SP0x86 hivelist
Volatility Foundation Volatility Framework 2.4
Virtual    Physical   Name
---------- ---------- ----
0x82b7a140 0x02b7a140 [no name]
0x820235c8 0x203675c8 \SystemRoot\System32\Config\SAM
0x87a1a250 0x27eb3250 \REGISTRY\MACHINE\SYSTEM
0x87a429d0 0x27f9d9d0 \REGISTRY\MACHINE\HARDWARE
0x87ac34f8 0x135804f8 \SystemRoot\System32\Config\DEFAULT
0x88603008 0x20d36008 \??\C:\Windows\ServiceProfiles\NetworkService\NTUSER.DAT
0x88691008 0x1ca1c008 \??\C:\Windows\ServiceProfiles\LocalService\NTUSER.DAT
0x9141e9d0 0x1dc569d0 \??\C:\Windows\System32\config\COMPONENTS
[snip]
```

---

**NOTE**

Notice that in addition to well-known registries (`SAM`, `SYSTEM`, `NTUSER.DAT`) there is also a registry that has a name `[no name]`. This registry contains keys and symbolic links for `REGISTRY\A` (an application hive), `REGISTRY\MACHINE` and `REGISTRY\USER`. For information regarding application hives, see [http://msdn.microsoft.com/en-us/library/windows/hardware/jj673019%28v=vs.85%29.aspx](http://msdn.microsoft.com/en-us/library/windows/hardware/jj673019%28v=vs.85%29.aspx).

---

Locating registry hives is critical because your ability to print the actual key and value data relies on first finding the hives. The registry file format is well documented by Timothy D. Morgan ([http://sentinelchicken.com/data/TheWindowsNTRegistryFileFormat.pdf](http://sentinelchicken.com/data/TheWindowsNTRegistryFileFormat.pdf)). You can see the simplified structure of a registry file on disk in [Figure 10-2](#figure10-2). The registry file contains a header and is broken up into sections called hive bins. Subsequently, each hive bin has a header and is broken up into cells. The cells contain the actual key and value data.

![c10f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c10/c10f002.png)

*[**Figure 10-2:**](#figureanchor10-2) A simplified diagram of the registry file format*

### Address Translations

Because of address translation, things become a bit more complicated when dealing with registry hives in memory as opposed to registry files on disk. The Configuration Manager (CM) is the component of the kernel that manages the registry ([http://msdn.microsoft.com/](http://msdn.microsoft.com/)en-us/library/windows/hardware/ff565712%28v=vs.85%29.aspx) and specifically deals with the address translation. The CM creates a mapping between cell indexes (values that are used to find the cells that contain the registry key data) and virtual addresses. The CM then stores this mapping in the `_HHIVE` structure. The member of interest is `Storage`, which is of type `_DUAL`. If you look up the `_DUAL` structure, you will see the `Map` member.

```
>>> dt("_DUAL")
'_DUAL' (220 bytes)
0x0   : Length               ['unsigned long']
0x4   : Map                  ['pointer', ['_HMAP_DIRECTORY']]
0x8   : SmallDir             ['pointer', ['_HMAP_TABLE']]
0xc   : Guard                ['unsigned long']
0x10  : FreeDisplay          ['array', 24, ['_RTL_BITMAP']]
[snip]
```

If you follow the `Map` member, you find all the structures needed to correctly obtain the virtual address for a registry key (in bold):

```
>>> dt("_HMAP_DIRECTORY")
'_HMAP_DIRECTORY' (4096 bytes)
0x0   : Directory            ['array', 1024, ['pointer', ['_HMAP_TABLE']]]

>>> dt("_HMAP_TABLE")
'_HMAP_TABLE' (8192 bytes)
0x0   : Table                ['array', 512, ['_HMAP_ENTRY']]

>>> dt("_HMAP_ENTRY")
'_HMAP_ENTRY' (16 bytes)
0x0   : BlockAddress         ['unsigned long']
0x4   : BinAddress           ['unsigned long']
0x8   : CmView               ['pointer', ['_CM_VIEW_OF_FILE']]
0xc   : MemAlloc             ['unsigned long']
```

Every cell index is broken up and used as a set of indices into the aforementioned structures. [Figure 10-3](#figure10-3) demonstrates an example of how cell indexes are broken up to obtain virtual addresses, which was described in Brendan Dolan-Gavitt’s presentation ([http://www.dfrws.org/2008/proceedings/p26-dolan-gavitt_pres.pdf](http://www.dfrws.org/2008/proceedings/p26-dolan-gavitt_pres.pdf)). Here is a description of each of the bit fields:

- **Bit 0:** Indicates whether the key is stable or volatile. Stable keys can also be found in the registry file on disk, whereas volatile keys are found only in memory.
- **Bits 1–10:** An index into the `Directory` member.
- **Bits 11–19:** An index into the `Table` member.
- **Bits 20–31:** The offset within the `BlockAddress` of where the key data resides. This is the cell within the registry. The cell contains the length of the data. Therefore, after you find the offset within the `BlockAddress`, you must add 4 (the size of the `Length` member) to get to the actual data.

![c10f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c10/c10f003.png)

*[**Figure 10-3:**](#figureanchor10-3) Cell index breakdown to obtain a virtual address*

### Printing Keys and Values

Registry keys are stored as a tree-like structure, where a root key exists. The children, or subkeys, are traversed until the leaf node (the last part of the key path) is accessed. Therefore, to access a registry key and its data, you have to start from the root key and walk down the tree until you reach the leaf node. The structure for nodes, `_CM_KEY_NODE`, is shown in the following code:

```
>>> dt("_CM_KEY_NODE")
'_CM_KEY_NODE' (80 bytes)
0x0   : Signature                 ['String', {'length': 2}]
0x2   : Flags                     ['unsigned short']
0x4   : LastWriteTime             ['WinTimeStamp', {}]
0xc   : Spare                     ['unsigned long']
0x10  : Parent                    ['unsigned long']
0x14  : SubKeyCounts              ['array', 2, ['unsigned long']]
0x1c  : ChildHiveReference        ['_CM_KEY_REFERENCE']
0x1c  : SubKeyLists               ['array', 2, ['unsigned long']]
0x24  : ValueList                 ['_CHILD_LIST']
[snip]
0x4c  : Name                      ['String', {
                   'length': <function <lambda> at 0x1017eb5f0>}]
```

When you use the `printkey` plugin, you pass it the desired registry key path on command line (the `–K/--key` argument). The plugin finds all available registries in memory and accesses the `SubKeyLists` and `ValueLists` members to traverse the trees. Thus, this plugin enables you to print a key, its subkeys, and its values. The following example shows you how to use this plugin:

```
$ python vol.py -f win7.vmem --profile=Win7SP1x86 printkey 
    -K "controlset001\control\computername"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile
----------------------------
Registry: \REGISTRY\MACHINE\SYSTEM
Key name: ComputerName (S)
Last updated: 2011-10-20 15:25:16 
Subkeys:
  (S) ComputerName
  (V) ActiveComputerName
Values:
```

In the output, you can see the registry path, key name, last write time, subkeys, and any values that the key has (in this case, there were none). The `printkey` plugin also tells you whether the registry key or its subkeys are stable (S) or volatile (V).

---

**NOTE**

You can also pass the `printkey` plugin an offset (using `–o/--offset`), which specifies the virtual address of a specific registry hive. This can be useful if you want to concentrate on only one registry hive. You obtain the virtual address from the `hivelist` plugin, as shown earlier in the chapter.

---

### Detecting Malware Persistence

Several registry keys are relevant in investigations involving malware. For example, malware may need a way to persist on a system even after the system is rebooted. One of the easiest ways to accomplish this task is to modify one of the startup registry keys. These keys contain information about programs that run when the system boots up or a user logs in. Therefore, you should check these known registry keys to see if the malware is using them to persist on the machine. The following list shows some known startup keys.

- For system startup: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`
- For user logons: `HKCU\Software\Microsoft\Windows NT\CurrentVersion\Windows HKCU\Software\Microsoft\Windows NT\CurrentVersion\Windows\Run HKCU\Software\Microsoft\Windows\CurrentVersion\Run HKCU\Software\Microsoft\Windows\CurrentVersion\RunOnce`

For a more comprehensive list of startup keys, see the `RegRipper` wiki ([https://code.google.com/p/regripper/wiki/ASEPs](https://code.google.com/p/regripper/wiki/ASEPs)) or the Sysinternals `AutoRuns` utility ([http://technet.microsoft.com/en-us/sysinternals/bb963902.aspx](http://technet.microsoft.com/en-us/sysinternals/bb963902.aspx)).

---

**NOTE**

`HKEY_CURRENT_USER` (or `HKCU` for short) refers to a user-specific registry. `HKEY_LOCAL_MACHINE` (or `HKLM` for short) refers to a registry used by the system.

---

An example of malware persistence is shown in the following output. The malicious executable `C:\WINDOWS\system32\svchosts.exe` is run every time the system starts up. This is immediately suspicious because no `svchosts.exe` executable exists on a clean Windows machine—it is attempting to blend in with the legitimate `svchost.exe` (without the extra “s”). Notice that you do not have to prefix the `–K/--key` argument with `HKLM\SOFTWARE` because it is actually not part of the path within the registry, but instead denotes which registry contains the key (for example, the `SOFTWARE` hive on the local machine).

```
$ python vol.py -f grrcon.raw --profile=WinXPSP3x86 printkey 
      -K "Microsoft\Windows\CurrentVersion\Run"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\software
Key name: Run (S)
Last updated: 2012-04-28 01:59:22 UTC+0000

Subkeys:
  (S) OptionalComponents

Values:
REG_SZ        Adobe Reader Speed Launcher : 
        (S) "C:\Program Files\Adobe\Reader 9.0\Reader\Reader_sl.exe"
REG_SZ        Adobe ARM       : 
        (S) "C:\Program Files\Common Files\Adobe\ARM\1.0\AdobeARM.exe"
REG_SZ        svchosts        : 
        (S) C:\WINDOWS\system32\svchosts.exe
```

The following output shows an example of persistence when the user logs in. In this case, the program of interest turns out to be a key logger that runs every time the user `Andrew` (as shown in the registry path) logs on to the system. Notice that the entire path after `HKCU` is given to `printkey`:

```
$ python vol.py -f Win7.raw --profile=Win7SP1x64 printkey 
      -K "SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN" 
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable  (V) = Volatile
---------------------------- 
Registry: \??\C:\Users\Andrew\ntuser.dat 
Key name: Run (S) 
Last updated: 2013-03-10 22:47:09 UTC+0000

Subkeys:

Values:
REG_SZ mswinnt : (S) "C:\Users\Andrew\Desktop\mswinnt.exe" 
      --logfile=log.txt --encryption-index=4
```

Another method of persistence that malware often uses is the creation of a service. When a service is created, the registry (in particular, `HKLM\SYSTEM\CurrentControlSet\Services`) is modified to contain information about the service. You can print this key and determine whether a service name stands out as suspicious. If you use timelines, as discussed in Chapter 18, and examine the services registry key, you can quickly identify newly added services based on the last written timestamps. The following is example output from a memory sample with Stuxnet that demonstrates this persistence mechanism:

```
$ python vol.py –f stuxnet.vmem --profile=WinXPSP3x86 printkey 
      -K "ControlSet001\services\MRxNet"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\system
Key name: MRxNet (S)
Last updated: 2011-06-03 04:26:47 UTC+0000

Subkeys:
  (V) Enum

Values:
REG_SZ        Description     : (S) MRXNET
REG_SZ        DisplayName     : (S) MRXNET
REG_DWORD     ErrorControl    : (S) 0
REG_SZ        Group           : (S) Network
REG_SZ        ImagePath       : (S) \??\C:\WINDOWS\system32\Drivers\mrxnet.sys
REG_DWORD     Start           : (S) 1
REG_DWORD     Type            : (S) 1
```

---

**NOTE**

You can obtain the `CurrentControlSet` by querying the following volatile key:

```
$ vol.py -f XPSP3x86.vmem --profile=WinXPSP3x86 printkey 
      -K currentcontrolset
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\system
Key name: CurrentControlSet (V)
Last updated: 2010-10-29 17:08:47 UTC+0000

Subkeys:

Values:
REG_LINK      SymbolicLinkValue : (V) \Registry\Machine\System\ControlSet001
```

This is important to note because there can be many control sets that contain different system configurations (see [http://support.microsoft.com/kb/100010](http://support.microsoft.com/kb/100010)). The `CurrentControlSet` contains the settings that the machine is using at the current moment. Therefore, if you use the incorrect control set, you may not see the current configurations.

---

Although the `printkey` plugin is very useful, it is also limited in that it prints only the raw key values. This is fine for integer or string values, but it is not sufficient for keys that contain binary or embedded data, such as Userassist keys. These keys require some extra processing to interpret before displaying data to the user; otherwise, it would just look like a blob of hex bytes. Also, the `printkey` plugin only checks one registry key at a time. For these reasons, the Volatility Registry API was created.

## Volatility’s Registry API

The Registry API was designed to allow easy processing of complicated registry keys or many keys at the same time. For example, you can use it to automatically check the 20 most common startup keys, sort keys based on their last write time, and so on. In the following code, we show you how to import and instantiate the Registry API from the `volshell` plugin. You use nearly identical code to call the API from your own plugins as well.

```
>>> import volatility.plugins.registry.registryapi as registryapi
>>> regapi = registryapi.RegistryApi(self._config)
```

When the Registry API object is instantiated, a dictionary of all registry files in the memory sample is saved. This makes it more efficient to switch between hives without rescanning. You can check out the `RegRipper` project ([http://code.google.com/p/regripper/](http://code.google.com/p/regripper/)) to get an idea of the numerous possibilities for writing plugins. In fact, Brendan created a proof of concept project named `VolRip` for an older version of Volatility that allowed an investigator to run `RegRipper` commands against memory-resident registry hives (see [http://moyix.blogspot.com/2009/03/regripper-and-volatility-prototype.html](http://moyix.blogspot.com/2009/03/regripper-and-volatility-prototype.html)). However, this was replaced with the Registry API, which is more portable because it doesn’t rely on the Perl-to-Python glue.

---

**NOTE**

An alternative to parsing the registry files in memory is using the `dumpfiles` plugin (described in Chapter 16) to extract hives from the Windows cache manager and then parse them with external tools. This method pulls the cached copy of the hive file, so volatile keys will not be included. Furthermore, there may be zero-padded gaps in each registry file due to paging. Most offline registry tools expect to parse a complete registry file from disk (not one dumped from memory) and may need some tweaking to handle these dumped registry files.

Also note that Windows 7 systems do not cache the registry hive files in the same manner as earlier versions of Windows. Thus, `dumpfiles` cannot be used in the described manner.

---

The following shows an example from within the `volshell` plugin that uses the Registry API to print out the subkeys of a designated registry key. First you set the current context to be the `NTUSER.DAT` registry hive of the administrator, and then you use the `reg_get_all_subkeys` function. In this case, you just print out the name, but you could process the results as you would any registry key of type `_CM_KEY_NODE`:

```
>>> regapi.set_current(hive_name = "NTUSER.DAT", user = "administrator")
>>> key = "software\\microsoft\\windows\\currentversion\\explorer"
>>> for subkey in regapi.reg_get_all_subkeys(None, key = key):
...     print subkey.Name
...
Advanced
BitBucket
CabinetState
CD Burning
CLSID
ComDlg32
[snip]
```

Additionally, the following code snippet shows how to obtain registry values. Again, this is from within the `volshell` plugin. If you want to get a particular value by name, you can use `reg_get_value`.

```
>>> k = "controlset001\\Control\\ComputerName\\ComputerName"
>>> v = "ComputerName"
>>> val = regapi.reg_get_value(hive_name = "system", key = k, value = v)
>>> print val
BOB-DCADFEDC55C
```

The following code shows how to print multiple registry values. Here you can see that one of the startup keys is used, and each of its values is printed. Notice that you can see the malicious program that runs every time the system reboots:

```
>>> k = "Microsoft\\Windows\\CurrentVersion\\Run"
>>> regapi.set_current(hive_name = "software")
>>> for value, data in regapi.reg_yield_values(hive_name = "software", key = k):
...     print value, "\n      ", data
... 
Adobe Reader Speed Launcher 
      "C:\Program Files\Adobe\Reader 9.0\Reader\Reader_sl.exe"
Adobe ARM 
      "C:\Program Files\Common Files\Adobe\ARM\1.0\AdobeARM.exe"
svchosts 
      C:\WINDOWS\system32\svchosts.exe
```

If you wanted to get the last ten modified keys from the administrator’s `NTUSER.DAT` registry, the following code snippet shows how to accomplish that. In this case, the last activity in the `NTUSER.DAT` hive shows that a network share was created:

```
>>> hive = "NTUSER.DAT"
>>> for t, k in regapi.reg_get_last_modified(hive_name = hive, count = 10):
...     print t, k
... 
2012-04-28 02:22:16 UTC+0000  
  $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer
2012-04-28 02:21:41 UTC+0000
  $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
    \##DC01#response
2012-04-28 02:21:41 UTC+0000 
  $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
2012-04-28 02:21:41 UTC+0000 $$$PROTO.HIV\Network\z
2012-04-28 02:21:41 UTC+0000 $$$PROTO.HIV\Network
2012-04-28 02:21:41 UTC+0000 $$$PROTO.HIV
2012-04-28 02:21:21 UTC+0000 
  $$$PROTO.HIV\Software\Microsoft\Windows NT\CurrentVersion\PrinterPorts
2012-04-28 02:21:21 UTC+0000 
  $$$PROTO.HIV\Software\Microsoft\Windows NT\CurrentVersion\Devices
2012-04-28 02:21:16 UTC+0000 $$$PROTO.HIV\SessionInformation
2012-04-28 02:21:15 UTC+0000 
  $$$PROTO.HIV\Software\Microsoft\Windows\ShellNoRoam\MUICache
```

If you want to see the subkeys and values for each of these keys, you can use a combination of the functions you’ve just seen. The following is example code and partial output, in which you can see that the network share [\\DC01\response](http://\\DC01\response) was mapped to drive “z”. You can use the `consoles` plugin (see Chapter 17) to look for `net use` commands and see whether the drive was mapped by the attacker or by the person who collected the memory sample.

```
>>> hive = "NTUSER.DAT"
>>> for t, k in regapi.reg_get_last_modified(hive_name = hive, count = 10):
...     print "LastWriteTime:", t
...     print "Key:", k
...     k = k.replace("$$$PROTO.HIV\\", "")
...     for subkey in regapi.reg_get_all_subkeys(hive_name = hive, key = k):
...             print "Subkey: ", subkey.Name
...     for value, data in regapi.reg_yield_values(hive_name = hive, key = k):
...             print "Value:", value, data
...     print "*" * 20
...
[snip]
LastWriteTime: 2012-04-28 02:21:41 UTC+0000 
Key: $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer
 \MountPoints2\##DC01#response
Value: BaseClass Drive
Value: _CommentFromDesktopINI 
Value: _LabelFromDesktopINI 
[snip]
********************
LastWriteTime: 2012-04-28 02:21:41 UTC+0000 
Key: $$$PROTO.HIV\Network\z
Value: RemotePath \\DC01\response
Value: UserName 
Value: ProviderName Microsoft Windows Network
Value: ProviderType 131072
Value: ConnectionType 1
Value: DeferFlags 4
```

## Parsing Userassist Keys

The Userassist keys are an important registry artifact used for determining what programs the user ran, as well as the time they were run. These keys are found in the `NTUSER.DAT` registries of each user on a machine. All the information is contained in a binary blob that must be parsed in a special way. The key paths may be different depending on the system you are investigating. For example, on Windows XP, 2003, Vista, and 2008, the Userassist key path is this:

```
HKCU\software\microsoft\windows\currentversion\explorer\userassist
  \{75048700-EF1F-11D0-9888-006097DEACF9}\Count
```

Starting in Windows 7, the Userassist key path can be one of the following:

```
HKCU\software\microsoft\windows\currentversion\explorer\userassist
  \{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}\Count
HKCU\software\microsoft\windows\currentversion\explorer\userassist
  \{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}\Count
```

In addition to the binary data that must be parsed for each of these keys, the value name contains the path of the program (or link) that was accessed. However, it is rot13 encoded, which is a simple Caesar cipher in which the letters are shifted by 13 places. The following raw data was extracted with the `printkey` plugin. The value, as you can see, is not readable because it is rot13 encoded. Furthermore, the binary data contains a timestamp in bold.

```
REG_BINARY    HRZR_EHACNGU:P:\JVAQBJF\flfgrz32\pzq.rkr : (S) 
 0x00000000  01 00 00 00 06 00 00 00 b0 41 5e b0 95 b6 ca 01 
```

Parsing Userassist data utilizes defined structures. Similar to the key paths, the structures vary depending on the operating system. You can see the structure for Windows XP, 2003, Vista, and 2008 machines in the following code. The members of interest are `CountStartingAtFive`, which is the number of times the application has run, and the `LastUpdated` timestamp, which is the last time the application was run.

```
>>> dt("_VOLUSER_ASSIST_TYPES")
'_VOLUSER_ASSIST_TYPES' (16 bytes)
0x0   : ID                             ['unsigned int']
0x4   : CountStartingAtFive            ['unsigned int']
0x8   : LastUpdated                    ['WinTimeStamp']
```

The following output shows the translated data from the `userassist` plugin. You can see the path to the program (`cmd.exe`) and determine that it ran one time, at 3:42:15 on February 26, 2010. Based on this information, you can go on to use the `cmdscan` or `consoles` plugins (see Chapter 17) to see whether any attacker’s commands still reside in memory. The `userassist` plugin also outputs the raw binary data in case you need to verify that the output is correct.

```
$ python vol.py –f XPSP3x86.vmem --profile=WinXPSP3x86 userassist
[snip]
REG_BINARY    UEME_RUNPATH:C:\WINDOWS\system32\cmd.exe : 
ID:             1
Count:          1
Last updated:   2010-02-26 03:42:15 
0x00000000  01 00 00 00 06 00 00 00 b0 41 5e b0 95 b6 ca 01   
```

## Detecting Malware with the Shimcache

The Shimcache registry keys are part of the Application Compatibility Database, which “identifies application compatibility issues and their solutions” (see [http://msdn.microsoft.com/en-us/library/bb432182(v=vs.85).aspx](http://msdn.microsoft.com/en-us/library/bb432182(v=vs.85).aspx)). These keys contains a path for an executable and the last modified timestamp from the `$STANDARD_INFORMATION` attribute of the MFT entry. This is very useful for proving that a piece of malware was on the system and what time it ran. Two possible registry keys are used, depending on the operating system.

- For Windows XP: `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\AppCompatibility`
- For Window 2003, Vista, 2008, 7, and 8: `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\AppCompatCache`

If you print out those registry keys, however, you will see a lot of binary data. This data must be parsed using specific structures. The following code shows the structures used to represent the Shimcache records on a Windows XP system:

```
>>> dt("ShimRecords")
'ShimRecords' (None bytes)
0x0   : Magic               ['unsigned int']
0x8   : NumRecords          ['short']
0x190 : Entries             ['array', <function <lambda> at 0x103413488>, 
                                ['AppCompatCacheEntry']]

>>> dt("AppCompatCacheEntry")
'AppCompatCacheEntry' (552 bytes)
0x0   : Path                ['NullString', {'length': 520, 'encoding': 'utf8'}]
0x210 : LastModified        ['WinTimeStamp', {}]
0x218 : FileSize            ['long long']
0x220 : LastUpdate          ['WinTimeStamp', {}]
```

A member of interest is the `Entries` list of `ShimRecords`, which is a list of `AppCompatCacheEntry` objects. The `AppCompatCacheEntry` objects are the actual objects that contain the information about the Shimcache record, such as the file `Path` and timestamps. The following output shows the raw data for the `AppCompatCache` value on a Windows XP system:

```
$ python vol.py –f XPSP3x86.vmem --profile=WinXPSP3x86 printkey 
      –K "ControlSet001\Control\Session Manager\AppCompatibility"
[snip]
REG_BINARY    AppCompatCache  : (S) 
[snip]
0x00000190  5c 00 3f 00 3f 00 5c 00 43 00 3a 00 5c 00 57 00   \.?.?.\.C.:.\.W.
0x000001a0  49 00 4e 00 44 00 4f 00 57 00 53 00 5c 00 73 00   I.N.D.O.W.S.\.s.
0x000001b0  79 00 73 00 74 00 65 00 6d 00 33 00 32 00 5c 00   y.s.t.e.m.3.2.\.
0x000001c0  6f 00 6f 00 62 00 65 00 5c 00 6d 00 73 00 6f 00   o.o.b.e.\.m.s.o.
0x000001d0  6f 00 62 00 65 00 2e 00 65 00 78 00 65 00 00 00   o.b.e...e.x.e...
[snip]
0x000003a0  00 a0 13 80 5e 3c c6 01 00 6e 00 00 00 00 00 00   ....^<...n......
0x000003b0  bc d9 7b 22 94 b6 ca 01 5c 00 3f 00 3f 00 5c 00   ..{"....\.?.?.\.
[snip]
```

The following example shows (redacted) output from a Windows 2003 server. As you can see, some mysteriously named executables are present:

```
$ python vol.py –f PhysicalMemory.001 --profile=Win2003SP2x86 shimcache
Volatility Foundation Volatility Framework 2.4
Last Modified                  Path
------------------------------ ----
[snip]
2007-02-17 10:19:26 UTC+0000   \??\C:\WINDOWS\system32\inetsrv\iisrstas.exe
2007-02-17 10:19:26 UTC+0000   \??\C:\WINDOWS\system32\iisreset.exe
2007-02-17 10:59:04 UTC+0000   \??\C:\Program Files\Outlook Express\setup50.exe
2009-03-08 11:32:52 UTC+0000   \??\C:\WINDOWS\system32\ieudinit.exe
2010-07-22 07:47:49 UTC+0000   \??\C:\XXX\nv.exe
2010-07-22 08:40:57 UTC+0000   \??\C:\XXX\123.exe
2010-07-22 07:44:57 UTC+0000   \??\C:\XXX\dl.exe
2010-07-22 07:46:41 UTC+0000   \??\C:\XXX\ow.exe
[snip]
2010-02-06 23:45:26 UTC+0000   \??\C:\WINDOWS\PSEXESVC.EXE
[snip]
2010-01-19 09:21:41 UTC+0000   \??\E:\XXX\sample.exe
2010-01-19 09:02:26 UTC+0000   \??\E:\XXX\s.exe
[snip]
```

---

**NOTE**

One thing to note about the Shimcache entries is that older entries are often overwritten with newer ones as the maximum size of the key value is reached (the size varies depending on the system). Therefore, you may see residual data (similar to slack space on a hard disk) on a machine that has been running for a while.

---

## Reconstructing Activities with Shellbags

Shellbags is a commonly used term to describe a collection of registry keys that allow the “Windows operating system to track user window viewing preferences specific to Windows Explorer” (see [http://www.dfrws.org/2009/proceedings/p69-zhu.pdf](http://www.dfrws.org/2009/proceedings/p69-zhu.pdf)). These keys contain a wealth of information relevant for a forensic investigation. Some example artifacts that you can find include these:

- Windows sizes and preferences
- Icon and folder view settings
- Metadata such as MAC timestamps
- Most Recently Used (MRU) files and file type (zip, directory, installer)
- Files, folders, zip files, and installers that existed at one point on the system (even if deleted)
- Network shares and folders within the shares
- Metadata associated with any of these types that may include timestamps and absolute paths
- Information about TrueCrypt volumes

---

**NOTE**

For more information on the Shellbag data structures, the corresponding registry keys, and their use in forensic investigations, see the following resources:

- *Shellbag Analysis* by Harlan Carvey: [http://windowsir.blogspot.com/2012/08/shellbag-analysis.html](http://windowsir.blogspot.com/2012/08/shellbag-analysis.html)
- *Windows Shellbag Forensics* by Willi Ballenthin: [http://www.williballenthin.com/forensics/shellbags](http://www.williballenthin.com/forensics/shellbags)
- *Shellbags Forensics: Addressing a Misconception:* [http://www.4n6k.com/2013/12/shellbags-forensics-addressing.html](http://www.4n6k.com/2013/12/shellbags-forensics-addressing.html)

---

### Shellbags in Memory

The `shellbags` plugin for Volatility uses the Registry API to extract data from the appropriate keys. It then parses that data using Shellbag data types and outputs the formatted version, along with the MRU information. Using the MRU details, you can correlate the last write time of the registry key with the last updated `shellbags` item. Here is an example of using the `shellbags` plugin:

```
$ python vol.py –f XPSP3.vmem --profile=WinXPSP3x86 shellbags
Volatility Foundation Volatility Framework 2.4
Registry: \Device\HarddiskVolume1\Documents and Settings\User\NTUSER.DAT 
Key: Software\Microsoft\Windows\ShellNoRoam\BagMRU\0\0
Last updated: 2011-06-03 04:24:36 
Value:          1
Mru:            0
File Name:      DOCUME~1       
Modified Date:  2010-08-22 17:38:04  
Create Date:    2010-08-22 13:32:26  
Access Date:    2010-08-26 01:04:52  
File Attribute: DIR
Path:           C:\Documents and Settings
------------------------------------------------------------
Value:          0
Mru:            1
File Name:      PROGRA~1       
Modified Date:  2010-08-25 23:04:02   
Create Date:    2010-08-22 13:32:48  
Access Date:    2010-08-25 23:04:22  
File Attribute: RO, DIR
Path:           C:\Program Files
------------------------------------------------------------
Value:          4
Mru:            2
File Name:      WINDOWS        
Modified Date:  2010-08-26 00:06:24  
Create Date:    2010-08-22 13:29:34  
Access Date:    2010-10-08 03:27:40  
File Attribute: DIR
Path:           C:\WINDOWS
[snip]
```

Here are a few things to note about Shellbags entries:

- `SHELLITEM` entries remain in the registry even after the file has been deleted.
- Timestamps associated with `SHELLITEM` entries are not updated, even if the file is modified or accessed sometime later.
- `ITEMPOS` entries are updated if the file is moved, deleted, or accessed.
- If a user is not logged on to the system at the time the memory sample is taken, that user’s hives are not available in memory and therefore the Shellbag data is not processed.

### Finding TrueCrypt Volumes with Shellbags

TrueCrypt volumes also appear in Shellbags keys, often as `ITEMPOS` entries. Because `ITEMPOS` entries are updated, if the TrueCrypt volume is moved or deleted, its entry will be updated or removed to reflect the change. Files accessed from this volume will have their Shellbags entries remain intact, however. In the following output, you can see that the machine has a TrueCrypt volume mounted on the `T:` drive, and it was accessed on `2012-09-25 11:48:46`.

```
$ python vol.py –f XPSP3.vmem --profile=WinXPSP3x86 shellbags
Volatility Foundation Volatility Framework 2.4
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT
Key: Software\Microsoft\Windows\ShellNoRoam\BagMRU\0
Last updated: 2012-09-25 13:22:43
Value   Mru   Entry Type     Path
------- ----- -------------- ----
    1       0     Volume Name    C:\
    3       1     Volume Name    Z:\
    4       2     Volume Name    T:\
***************************************************************************
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT 
Key: Software\Microsoft\Windows\ShellNoRoam\Bags\52\Shell
Last updated: 2012-09-25 12:51:28 
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      UserData       
Modified Date:  2012-06-22 19:28:50  
Create Date:    2012-06-22 19:28:50  
Access Date:    2012-09-25 12:51:18  
File Attribute: SYS, DIR
Path:           UserData       
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      RECENT~1.XBE   
Modified Date:  2010-10-18 14:00:50  
Create Date:    2010-10-18  14:00:50  
Access Date:    2010-10-18 14:00:50  
File Attribute: ARC
Path:           .recently-used.xbel
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      MYTRUE~1       
Modified Date:  2012-08-17 14:13:48  
Create Date:    2012-08-17 14:12:18  
Access Date:    2012-09-25 11:48:46  
File Attribute: RO, DIR
Path:           MyTrueCryptVolume
------------------------------------------------------------
```

---

**NOTE**

Note that the name `MyTrueCryptVolume` was chosen on purpose when the volume was created so that it would stand out in the registry. Real TrueCrypt volumes will probably not be so obviously named.

---

After the TrueCrypt volume is deleted, its `ITEMPOS` entry disappears from the registry, as shown by the following output:

```
$ python vol.py –f XPSP3.vmem --profile=WinXPSP3x86 shellbags
Volatility Foundation Volatility Framework 2.4
***************************************************************************
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT 
Key: Software\Microsoft\Windows\ShellNoRoam\Bags\52\Shell
Last updated: 2012-09-25 14:31:53 
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      UserData       
Modified Date:  2012-06-22 19:28:50  
Create Date:    2012-06-22 19:28:50  
Access Date:    2012-09-25 12:51:18  
File Attribute: SYS, DIR
Path:           UserData       
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      RECENT~1.XBE   
Modified Date:  2010-10-18 14:00:50  
Create Date:    2010-10-18  14:00:50  
Access Date:    2010-10-18 14:00:50  
File Attribute: ARC
Path:           .recently-used.xbel
------------------------------------------------------------
```

Although the `MyTrueCryptVolume` entry is no longer in the registry, any individual files that were accessed from the TrueCrypt volume while it was mounted may still have entries in the registry. Because the link between the TrueCrypt volume and its files in the Shellbags keys was severed after its deletion, it becomes difficult to definitively say what files existed on the TrueCrypt volume, given that these keys show only the filename instead of the full path. However, if you have the timestamp of when the TrueCrypt volume existed on the system (either from a previous memory image, registry snapshot, or an MFT file from the disk), you can then extrapolate that knowledge to find `ITEMPOS` entries for files within that time period. In the following example, you can potentially associate `recent news.txt` and `customer emails.txt` with the TrueCrypt volume because they were accessed within about an hour of the access time for `MyTrueCryptVolume`:

```
$ python vol.py –f XPSP3.vmem --profile=WinXPSP3x86 shellbags
Volatility Foundation Volatility Framework 2.4
***************************************************************************
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT
Key: Software\Microsoft\Windows\ShellNoRoam\Bags\63\Shell
Last updated: 2012-09-25 15:49:32
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      RECENT~1.TXT   
Modified Date:  2012-09-25 15:49:16  
Create Date:    2012-08-17 14:15:02  
Access Date:    2012-09-25 12:49:16  
File Attribute: ARC
Path:           recent news.txt
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      CUSTOM~1.TXT    
Modified Date:  2012-06-18 19:52:32  
Create Date:    2012-08-17 14:15:18  
Access Date:    2012-09-25 12:52:10
File Attribute: ARC
Path:           customer emails.txt
------------------------------------------------------------
```

### Timestomping Registry Keys

It is important to note that registry key timestamps can be overwritten, or “stomped.” This anti-forensics technique hides objects from timeline-based analysis. Joakim Schicht wrote a proof-of-concept tool, `SetRegTime`, to illustrate this capability (see [http://code.google.com/p/mft2csv/wiki/SetRegTime](http://code.google.com/p/mft2csv/wiki/SetRegTime)). This tool effectively overwrites desired timestamps in registry keys by using the Windows API (specifically, `NtSetInformationKey`). As previously discussed, because the Windows API is used, changes are reflected in memory within the five-second flush time. The following example demonstrates output from the `shellbags` plugin after a registry key’s timestamp is overwritten with `SetRegTime`:

```
$ python vol.py –f XPSP3x86.vmem --profile=WinXPSP3x86 shellbags
[snip]
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT 
Key: Software\Microsoft\Windows\ShellNoRoam\Bags\63\Shell
Last updated: 3024-05-21 00:00:00 
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      NEWTEX~1.TXT   
Modified Date:  2012-08-17 14:14:56  
Create Date:    2012-08-17 14:14:50  
Access Date:    2012-09-25 11:49:38
File Attribute: ARC
Path:           New Text Document.txt
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      POISON~1.PY    
Modified Date:  2012-06-18 19:52:32  
Create Date:    2012-08-17 14:15:18  
Access Date:    2012-09-25 12:52:10
File Attribute: ARC
Path:           poison_ivy.py
------------------------------------------------------------
```

The output shows the `LastWriteTime` as `3024-05-21 00:00:00`, a date clearly in the future. Notice that this new date doesn’t have any effect on the embedded timestamps in the Shellbags entries (or any other registry keys with embedded timestamps discussed in this chapter). With Shellbags entries, you know that you should have at least one embedded timestamp that has the same date as the `LastWriteTime`, which is obviously not true in this case. Therefore, if you see `LastWriteTime` timestamps that are out of sync with the values of embedded timestamps, this is an obvious flag that something is wrong with that key.

If keys without embedded timestamps are chosen for timestomping, and if the new timestomped dates are within a timeframe that seems normal (for example, not in the year 3024), it is harder for you to discover whether the timestamps of those registry keys were actually changed. In these cases, you might have to use the timestamps of other system artifacts, in conjunction with registry key timestamps, to uncover these timestomped keys. To accomplish this, you can employ the methods discussed in Chapter 18, which covers creating thorough timelines, to expose such malicious activities.

## Dumping Password Hashes

You can dump account password hashes from memory samples using the `hashdump` plugin. The `hashdump` plugin uses keys from both the `SYSTEM` and `SAM` hives, which are found automatically using the Registry API. The hashes can then be fed to your hash-cracking tool to obtain the clear text password. This plugin is a favorite for the offensive community, as you can imagine.

As Brendan Dolan-Gavitt explains in his blog (see ([http://moyix.blogspot.com/2008/02/syskey-and-sam.html](http://moyix.blogspot.com/2008/02/syskey-and-sam.html)), generally two types of password hashes are stored in the `SAM`: the LanMan (LM) hash and the NT hash. The LM hash, which suffers from some design flaws that make it easy to crack, is considered obsolete. Thus, it is disabled by default on Windows Vista, 2008, 7, and 8. It can also explicitly be disabled on Windows XP and 2003 (see [http://www.microsoft.com/security/sir/strategy/default.aspx#!password_hashes](http://www.microsoft.com/security/sir/strategy/default.aspx#!password_hashes)). The NT hash, however, is supported by all modern Windows operating systems. The `hashdump` plugin obtains both types of hashes.

The following example demonstrates how to use the `hashdump` plugin to extract password hashes:

```
$ python vol.py -f  Bob.vmem --profile=WinXPSP3x86 hashdump 
Volatility Foundation Volatility Framework 2.4
Administrator:500:e52cac67419a9a2[snip]:8846f7eaee8fb117ad06bdd830b7586c:::
Guest:501:aad3b435b51404eeaad3b43[snip]:31d6cfe0d16ae931b73c59d7e0c089c0:::
HelpAssistant:1000:9f8ac2eaebcd2e[snip]:d95e38a172b3ddaa1ce0b63bb1f5e1fb:::
SUPPORT_388945a0:1002:aad3b435b51[snip]:ad052c1cbab3ec2502df165cd25d95bd::
```

After you have obtained the password hashes, you can use a password cracker such as John the Ripper ([http://www.openwall.com/john/](http://www.openwall.com/john/)), like so:

```
$ john hashes.txt
Loaded 6 password hashes with no different salts (LM DES [128/128 BS SSE2-16])
                 (SUPPORT_388945a0)
                 (Guest)
PASSWOR          (Administrator:1)
D                (Administrator:2)
[interrupted]

$ john --show hashes.txt
Administrator:PASSWORD:500:8846f7eaee8fb117ad06bdd830b7586c:::
Guest::501:31d6cfe0d16ae931b73c59d7e0c089c0:::
SUPPORT_388945a0::1002:ad052c1cbab3ec2502df165cd25d95bd:::
4 password hashes cracked, 2 left
```

You have the password, but it is in all uppercase letters, which may not be correct. If you use the community-enhanced “jumbo” version of John (see [http://insidetrust.blogspot.com/2011/01/password-cracking-using-john-ripper-jtr.html](http://insidetrust.blogspot.com/2011/01/password-cracking-using-john-ripper-jtr.html)), you can obtain the proper password, which just happens to be “password” (all lowercase) in this case:

```
$ john --show --format=LM hashes.txt | grep -v "password hashes" \
 | cut -d":" -f2 | sort -u > pass.txt

$ john --rules --wordlist=pass.txt --format=nt hashes.txt
Loaded 4 password hashes with no different salts (NT MD4 [128/128 X2 SSE2-16])
password         (Administrator)
guesses: 2  time: 0:00:00:00 DONE (Wed Jan  1 10:40:26 2014)  c/s: 3400  
      trying: Password3 - Passwording
Use the "--show" option to display all of the cracked passwords reliably
```

From an offensive view, this can be quite useful. For example, imagine that you have obtained access to a VMware ESX server with several virtual machines. You could then crack the passwords to each of the machines by accessing the snapshot memory files of each machine. Suddenly your attack space has substantially increased!

## Obtaining LSA Secrets

The `lsadump` plugin dumps decrypted LSA secrets from the registries of all supported Windows machines ([http://moyix.blogspot.com/2008/02/decrypting-lsa-secrets.html](http://moyix.blogspot.com/2008/02/decrypting-lsa-secrets.html)). This exposes information such as the default password (for systems with auto-login enabled), the RDP private key, and credentials used by Data Protection API (DPAPI ). The `lsadump` plugin uses both the `SYSTEM` and `SECURITY` hives, which are found automatically using the Registry API.

Some of the items that you can find in LSA Secrets include these:

- `$MACHINE.ACC`: Domain authentication ([http://support.microsoft.com/kb/175468)](http://support.microsoft.com/kb/175468)).
- `DefaultPassword`: Password used to log on to Windows when auto-login is enabled.
- `NL$KM`: Secret key used to encrypt cached domain passwords ([http://moyix.blogspot.com/2008/02/cached-domain-credentials.html](http://moyix.blogspot.com/2008/02/cached-domain-credentials.html)).
- `L$RTMTIMEBOMB_*`: Timestamp giving the date when an unactivated copy of Windows will stop working.
- `L$HYDRAENCKEY_*`: Private key used for Remote Desktop Protocol (RDP). If you also have a packet capture from a system that was attacked via RDP, you can extract the client’s public key from the packet capture and the server’s private key from memory; then decrypt the traffic.

You can see an example of the `lsadump` plugin in action in the following output, which shows the LSA Secret for the private RDP key:

```
$ python vol.py -f  XPSP3x86.vmem --profile=WinXPSP3x86 lsadump 
 Volatility Foundation Volatility Framework 2.4
 [snip]
 L$HYDRAENCKEY_28ada6da-d622-11d1-9cb9-00c04fb16e75
 0x00000000  52 53 41 32 48 00 00 00 00 02 00 00 3f 00 00 00   RSA2H.......?...
 0x00000010  01 00 01 00 f1 93 70 67 69 62 de d1 aa f0 99 67   ......pgib.....g
 0x00000020  83 bb 95 20 a0 de 05 a7 40 7b 7e 5e a9 d2 f5 bd   ........@{~^....
 0x00000030  52 37 18 c2 b5 6d f0 78 b3 cc 7e e0 b8 b7 70 01   R7...m.x..~...p.
 0x00000040  33 bf fb 3d 75 69 d8 e1 84 b4 ab b8 bc 82 63 d9   3..=ui........c.
 0x00000050  17 d3 80 d6 00 00 00 00 00 00 00 00 4d 40 cd 12   ............M@..
 0x00000060  c1 18 93 a6 ec a8 99 03 cb f7 76 ab bb 6d e8 63   ..........v..m.c
 [snip]
```

In the following output, you can see the LSA Secrets for the `DefaultPassword` and `DPAPI_SYSTEM`:

```
$ python vol.py -f Win7SP1x64.raw --profile=Win7SP1x64 lsadump
Volatility Foundation Volatility Framework 2.4
DefaultPassword
0x00000000  00 00 00 01 7e a3 eb 47 10 31 8b 1f 6b 54 65 5c   ....~..G.1..kTe\
0x00000010  23 67 b1 dd 03 00 00 00 00 00 00 00 3b 7e b7 96   #g..........;~..
0x00000020  d5 98 fa 71 32 24 24 b5 92 a0 8a cb 40 43 b5 24   ...q2$$.....@C.$
0x00000030  19 90 dd e3 15 96 f4 34 4e 8b 75 ea a0 49 b4 4f   .......4N.u..I.O
0x00000040  08 eb 90 ec e3 0a 7c 3d c7 87 f7 ef 3f 8a 5f ad   ......|=....?._.
0x00000050  c1 d7 f2 8f 01 99 98 c3 e1 8e 97 c9               ............
DPAPI_SYSTEM
0x00000000  00 00 00 01 7e a3 eb 47 10 31 8b 1f 6b 54 65 5c   ....~..G.1..kTe\
0x00000010  23 67 b1 dd 03 00 00 00 00 00 00 00 2b 04 ff 76   #g..........+..v
0x00000020  30 d3 c5 53 7b 8c 98 15 92 9b ab ec 68 83 7e cd   0..S{.......h.~.
0x00000030  f8 f8 17 6b ba 6a 68 f2 28 57 17 1a 89 1d f7 fd   ...k.jh.(W......
0x00000040  e9 97 32 fc a3 61 ce bc a1 3c 95 b6 d2 11 9b 98   ..2..a...<......
0x00000050  77 10 c9 fd 95 86 60 09 68 83 9f b0 38 ff 01 3c   w.....`.h...8..<
0x00000060  30 04 b5 47 8d eb 8c 85 2b 69 03 1b 60 67 9c 34   0..G....+i..`g.4
0x00000070  fa a5 0d 1f b5 eb 88 ea 82 92 28 40               ..........(@
```

## Summary

The registry is a core component of the Windows operating system and thus an important aspect of most digital investigations. Memory forensics enables an investigator to access the volatile parts of the registry that cannot be found on disk and discover registry modifications that may never get written back to disk. While an investigator can access cached versions of the traditional registry data that is typically stored within the file system, Volatility’s ability to analyze memory-resident registry artifacts introduces a new realm of analysis that isn’t possible with disk forensics. When you combine that power with structured analysis of the embedded data in Userassist, Shellbags, and Shimcache keys, you gain the ability to track many aspects of user activity. Additionally, by querying registry data in a memory dump, you can quickly locate malware persistence, cached passwords, and more!
