# Chapter 15  
 Windows GUI Subsystem, Part II

Part II of the Windows graphical user interface (GUI) subsystem analysis covers detection of message and event hooks, inspection of the `USER` object handle tables, extraction of data from the Windows clipboard, and various additional topics. You will also read through some in-depth case studies that leverage memory forensics and highlight the unique ability of the Volatility Framework to detect malicious code in RAM.

## Window Message Hooks

Applications can place hooks into the Windows GUI subsystem to customize the user experience, receive notification when certain actions take place, or record everything the user does—for example, to create a computer-based training (CBT) video. As you probably expected, this type of access and control is often exploited by malware to capture keystrokes, inject malicious dynamic link libraries (DLLs) into trusted processes, and perform other nefarious actions.

When a user presses a key, the system generates a `WM_KEYDOWN` message and delivers it (along with additional information, such as the exact key, whether `SHIFT` was down at the time, etc.) to the target window’s queue. The target window is usually the foreground window (in focus). When the message hits the queue, the thread that owns the window wakes up and processes the message—which could mean appending the typed character into a text edit field, taking some special action if the key is a “hot key,” or even just ignoring it. [Figure 15-1](#figure15-1) shows a very simplified diagram of a nonhooked messaging system.

![c15f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f001.png)

*[**Figure 15-1:**](#figureanchor15-1) A nonhooked window messaging system*

Message hooks can intercept the window messages before they reach the target window procedure. For example, attackers can spy on all keyboard-related messages, including `WM_KEYDOWN`, in order to log them, and then either pass them on to the intended application or prevent them from ever reaching the right place. This is one of the oldest and most effective ways to log keystrokes on Windows-based systems.

[Figure 15-2](#figure15-2) shows how the messaging system works when hooks are installed. When a message is generated, the DLL containing the hook procedure is mapped into to the address space of the specified thread(s) if it is not already loaded. The message is passed to the hook procedure, which handles it as desired; finally, if allowed, the message reaches the target window procedure for normal processing.

![c15f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f002.png)

*[**Figure 15-2:**](#figureanchor15-2) Malware can hook into the messaging system.*

### Message Hook Installation

Adversaries can use the `SetWindowsHookEx` function to install message hooks. The function prototype follows:

```
HHOOK WINAPI SetWindowsHookEx(
  _In_  int idHook,
  _In_  HOOKPROC lpfn,
  _In_  HINSTANCE hMod,
  _In_  DWORD dwThreadId
);
```

Here are descriptions for the parameters:

- `idHook` is one of the `WH_` constants, such as `WH_KEYBOARD` or `WH_MOUSE`. It specifies what types of messages should be monitored.
- `lpfn` is the address of a hook procedure that handles the message before the target window. This procedure is a function of type `HOOKPROC`, which receives the message, processes it, and optionally passes it to the next hook in the chain (or the target window if there are no other hooks) with `CallNextHookEx`.
- `hMod` is a handle for the DLL that contains the hook procedure. This DLL loads into the address space of the thread that owns the target window.
- `dwThreadId` is a thread ID (i.e., scope) for the hook. It can be a specific thread ID or 0 to affect all threads in the current desktop.

In almost all cases, malicious message hooks are global in scope. So during analysis, you will want to focus on global hooks and attempt to reconstruct the original parameters passed to `SetWindowsHookEx` so you can determine the offending DLL and hook function.

---

## **Analysis Objectives**

Your objectives are these:

- **Detect global hooks**: Determine whether any global (affecting all threads in the desktop) message hooks are installed.
- **DLL attribution**: Trace the hook function back to its owning DLL on disk.
- **Hook function analysis**: Analyze the hook function code to understand what UI interactions (keystrokes, mouse movements, etc.) are being monitored.

## **Data Structures**

The message hook structure is `tagHOOK`. An example from Windows 7 x64 follows:

```
>>> dt("tagHOOK")
'tagHOOK' (96 bytes)
0x0   : head                           ['_THRDESKHEAD']
0x28  : phkNext                        ['pointer64', ['tagHOOK']]
0x30  : iHook                          ['long']
0x38  : offPfn                         ['unsigned long long']
0x40  : flags                          ['Flags', {'bitmap': {'HF_INCHECKWHF': 8, 
  'HF_HOOKFAULTED': 4, 'HF_WX86KNOWNDLL': 6, 'HF_HUNG': 3, 'HF_FREED': 9, 
  'HF_ANSI': 1, 'HF_GLOBAL': 0, 'HF_DESTROYED': 7}}]
0x44  : ihmod                          ['long']
0x48  : ptiHooked                      ['pointer64', ['tagTHREADINFO']]
0x50  : rpdesk                         ['pointer64', ['tagDESKTOP']]
0x58  : fLastHookHung                  ['BitField', {'end_bit': 8, 
  'start_bit': 7, 'native_type': 'long'}]
0x58  : nTimeout                       ['BitField', {'end_bit': 7, 
  'start_bit': 0, 'native_type': 'unsigned long'}]
```

## **Key Points**

The key points are these:

- `head`: The common header for USER objects and can help identify the owning process or thread. More information on this is available in the “User Handles” section.
- `phkNext`: A pointer to the next hook in the chain. When a hook procedure calls `CallNextHookEx`, the system locates the next hook using this member.
- `offPfn`: A relative virtual address (RVA) to the hook procedure. The procedure can be in the same module as the code calling `SetWindowsHookEx` (for local thread-specific hooks only), in which case `ihmod` is -1. Otherwise, for global hooks, the procedure is in a DLL, and `ihmod` is an index into an array of atoms located at `win32k!_aatomSysLoaded`. To determine the name of the DLL, you must translate the `ihmod` into an atom and then obtain the atom name (see Chapter 14).
- `ptiHooked`: This value can be used to identify the hooked thread.
- `rpdesk`: Identifies the desktop in which the hook is set. Hooks cannot cross desktop boundaries.

---

### Detecting Message Hooks for DLL Injection

The `messagehooks` plugin enumerates *global* hooks by finding all desktops and accessing `tagDESKTOP.pDeskInfo.aphkStart`—an array of `tagHOOK` structures whose positions in the array indicate which type of message is to be filtered (such as `WH_KEYBOARD` or `WH_MOUSE`). The `tagDESKTOP.pDeskInfo.fsHooks` value is used as a bitmap to tell you which positions in the array are actively in use. Likewise, for each thread, the plugin scans for *local* (i.e., thread-specific) hooks by looking at `tagTHREADINFO.aphkStart` and `tagTHREADINFO.fsHooks`.

[Figure 15-3](#figure15-3) shows a disassembly of the Laqma malware installing a `WM_GETMESSAGE` hook. This is an example of malware using `SetWindowsHookEx` as simply a means to load DLLs in other processes instead of monitoring or intercepting messages. You can tell because the `lpfnWndProc` just passes control to the next hook in the chain by calling `CallNextHookEx`. Also note that the `dwThreadId` parameter is 0, which means the hook is global and will affect all GUI threads in the same desktop as the executing malware.

Running the `messagehooks` plugin on a memory dump infected with Laqma shows results like the following:

```
$ python vol.py -f laqma.vmem --profile=WinXPSP3x86 messagehooks --output=block
Volatility Foundation Volatility Framework 2.4
Offset(V)  : 0xbc693988
Session    : 0
Desktop    : WinSta0\Default
Thread     : <any>
Filter     : WH_GETMESSAGE
Flags      : HF_ANSI, HF_GLOBAL
Procedure  : 0x1fd9
ihmod      : 1
Module     : C:\WINDOWS\system32\Dll.dll

Offset(V)  : 0xbc693988
Session    : 0
Desktop    : WinSta0\Default
Thread     : 1584 (explorer.exe 1624)
Filter     : WH_GETMESSAGE
Flags      : HF_ANSI, HF_GLOBAL
Procedure  : 0x1fd9
ihmod      : 1
Module     : C:\WINDOWS\system32\Dll.dll

Offset(V)  : 0xbc693988
Session    : 0
Desktop    : WinSta0\Default
Thread     : 252 (VMwareUser.exe 1768)
Filter     : WH_GETMESSAGE
Flags      : HF_ANSI, HF_GLOBAL
Procedure  : 0x1fd9
ihmod      : 1
Module     : C:\WINDOWS\system32\Dll.dll
[snip]
```

![c15f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f003.png)

*[**Figure 15-3:**](#figureanchor15-3) Laqma installs a message hook to inject Dll.dll into other processes*

As you can see, all the hooks are global because the flags include `HF_GLOBAL`. That means they are the direct result of calling `SetWindowsHookEx` with the `dwThreadId` parameter set to 0. Although not all global hooks are malicious, out of the malware samples we have seen that hook window messages, they all use global hooks.

The difference between the three hooks shown is that the first one is global and was gathered from the `tagDESKTOP` structure. You can tell because the target thread is `<any>`. It tells you that any GUI threads that run in the `WinSta0\Default` desktop are subject to monitoring by the malware. The next two hooks are associated with specific threads (as a result of the global hook) and have caused the injection of `Dll.dll` into `explorer.exe` and `VMwareUser.exe`.

From the disassembly in [Figure 15-3](#figure15-3), you already know that the `lpfnWndProc` has no special payload; this hook exists only to inject a DLL into other processes. However, do not overlook the fact that the `messagehooks` plugin shows you the address (as an RVA) of the hook procedure in the DLL. In the examples shown, you can find the hook procedure at `0x1fd9` from the base of `Dll.dll` in the affected processes. Thus, if you did not preemptively know the purpose of a hook, you can easily use `volshell` and switch into the target process’ context and then disassemble the function.

As shown in the following code, you can locate the base address (`0xac0000`) of the injected DLL inside `explorer.exe` using the `dlllist` plugin. Next, you can disassemble the code at offset `0x1fd9`. Notice that the function consists of only a few instructions, which essentially just pass its arguments, in unmodified form, to `CallNextHookEx`.

```
$ python vol.py -f laqma.vmem --profile=WinXPSP3x86 dlllist -p 1624 
      | grep Dll.dll
Volatility Foundation Volatility Framework 2.4
0x00ac0000     0x8000 C:\Documents and Settings\Mal Ware\Desktop\Dll.dll

$ python vol.py -f laqma.vmem --profile=WinXPSP3x86 volshell
Volatility Foundation Volatility Framework 2.4
Current context: process System, pid=4, ppid=0 DTB=0x31a000
Welcome to volshell! 
To get help, type 'hh()'
>>> cc(pid = 1624)
Current context: process explorer.exe, pid=1624, ppid=1592 DTB=0x80001c0
>>> dis(0x00ac0000 + 0x00001fd9)
0xac1fd9 ff74240c                     PUSH DWORD [ESP+0xc]
0xac1fdd ff74240c                     PUSH DWORD [ESP+0xc]
0xac1fe1 ff74240c                     PUSH DWORD [ESP+0xc]
0xac1fe5 ff350060ac00                 PUSH DWORD [0xac6000]
0xac1feb ff157c40ac00                 CALL DWORD [0xac407c] ; CallNextHookEx
0xac1ff1 c20c00                       RET 0xc
```

The final artifact you should note regarding the use of message hooks for DLL injection is that the full path on disk to the malicious DLL is added to an atom table. As previously described in Chapter 14, you can inspect the atom tables with the `atoms` or `atomscan` plugins. In the output that follows, you should recognize the `Dll.dll` string:

```
$ python vol.py –f laqma.vmem --profile=WinXPSP3x86 atoms 
Volatility Foundation Volatility Framework 2.4
AtomOfs(V)       Atom Refs   Pinned Name
---------- ---------- ------ ------ ----
0xc10e000d10     0xc001      1      1 USER32
0xe155e958     0xc002      1      1 ObjectLink
0xe100a308     0xc003      1      1 OwnerLink
0xc15e018c00     0xc004      1      1 Native
0xe1b2aa88     0xc1b2      2      0 __axelsvc
0xe4bcb888     0xc1be      2      0 ShImgVw:CPreview
0xe11b0250     0xc1c1      2      0 __srvmgr32
0xe1f8bc30     0xc1c3      1      0 C:\WINDOWS\system32\psbase.dll
0xe28ed818     0xc1c7      1      0 BCGP_TEXT
0xc29e050c98     0xc19f      1      0 ControlOfs01420000000000FC
0xe11d6290     0xc1a0      1      0 C:\WINDOWS\system32\Dll.dll
0xe1106380     0xc1a1      1      0 BCGM_ONCHANGE_ACTIVE_TAB
0xe11a5090     0xc1a2      1      0 ControlOfs01EE0000000003C8
[snip]
```

---

**NOTE**

Not all DLL paths in the atom table are related to message hooks. Applications can put strings in the atom table for a number of reasons. For instance, `psbase.dll` appears right above `Dll.dll`, but `psbase.dll` is a legitimate component of Windows.

---

## User Handles

USER handles are references to objects in the GUI subsystem. Just as `CreateFile` returns a `HANDLE` to a `_FILE_OBJECT` managed by the NT executive object manager, `CreateWindow` returns an `HWND` (handle to a `tagWND)` that is managed by the GUI subsystem. There are either 20 (Windows XP through Vista) or 22 (Windows 7) types of USER objects, including `TYPE_FREE`.

From a malware and forensics perspective, the USER handle tables are extremely valuable because they provide an alternate method of finding evidence. For example, we already discussed how to find windows and hooks, but you can find the same objects by walking the USER handle table. Thus if an attacker tries to get creative and hide objects using DKOM, they must hide in two ways rather than one in order to be effective. Also, the USER handle tables can serve as a primary way of finding objects that you do not locate in other ways, such as event hooks and clipboard data.

---

**NOTE**

You may remember from Chapter 6 that each process’ `_EPROCESS.ObjectTable` points to a process-specific handle table of executive objects (files, registry keys, mutexes, etc.). The GUI subsystem is different: You have one USER handle table per session and all processes in the session share it. *That does not mean that you can access all objects in the USER handle table from any process*. As you will soon see, the system stores metadata with each handle to dictate which process or thread is the owner.

---

---

## **Analysis Objectives**

Your objectives are these:

- **Programmable API**: You can leverage the USER handle table’s API within Volatility to build plugins that analyze certain types of USER objects. This is how the `eventhooks` and `clipboard` plugins currently work.
- **Verification or cross-reference**: Malware may use rootkit techniques to hide handles to objects in various ways, but you can always check the USER handle table for an authoritative view of what resources are available. The USER handle table can also be manipulated, given administrator access and knowledge of the mostly undocumented underlying kernel data structures.

## **Data Structures**

Various structures are involved in the handle table. First, the `win32k!_gahti` symbol is an array of `tagHANDLETYPEINFO` structures—one for each object type. These structures are similar in concept to `nt!_OBJECT_TYPE` for executive objects. In particular, the handle type information structures tell you what pool tags are associated with each object type; whether the objects are allocated from the desktop heap, shared heap, or session pool; and whether the object is thread-owned or process-owned.

The structure for Windows 7 x64 is shown in the following code:

```
>>> dt("tagHANDLETYPEINFO")
'tagHANDLETYPEINFO' (16 bytes)
0x0   : fnDestroy                      ['pointer', ['void']]
0x8   : dwAllocTag                     ['String', {'length': 4}]
0xc   : bObjectCreateFlags             ['Flags', {'target': 'unsigned char', 
   'bitmap': {'OCF_VARIABLESIZE': 7, 'OCF_DESKTOPHEAP': 4, 
   'OCF_THREADOWNED': 0, 'OCF_SHAREDHEAP': 6, 'OCF_USEPOOLIFNODESKTOP': 5, 
   'OCF_USEPOOLQUOTA': 3, 'OCF_MARKPROCESS': 2, 'OCF_PROCESSOWNED': 1}}
```

## **Key Points**

The key points are these:

- `fnDestroy`: Points to the default deallocation/cleanup function for the object type.
- `dwAllocTag`: This value is similar to a pool tag; it consists of four ASCII characters that directly precede objects in memory, so you can use it to find or identify the allocations.
- `bObjectCreateFlags`: Flags that tell you whether an object is thread-owned or process-owned, whether it is allocated from the shared or desktop heaps, and so on.

---

### Enumerating USER Object Types

Take a quick look at the output of the `gahti` plugin on an x64 Windows 7 system. This plugin finds and parses the `win32k!_gahti` (gahti stands for *global array of handle table information*) to show you what types of objects you can expect to find on your target operating system.

The first entry in the gahti is always for `TYPE_FREE`, whose members are all 0. Once a handle is no longer in use, it is not immediately removed from the handle table. Instead, its type is simply set to `TYPE_FREE`, which gives you the opportunity to potentially recover information on previously used handles.

```
$ python vol.py -f win7x64cmd.dd --profile=Win7SP1x64 gahti
Volatility Foundation Volatility Framework 2.4
Session  Type                 Tag      fnDestroy          Flags
-------- -------------------- -------- ------------------ -----
       0 TYPE_FREE                     0x0000000000000000 
       0 TYPE_WINDOW          Uswd     0xfffff9600014f660 OCF_DESKTOPHEAP, 
OCF_THREADOWNED, OCF_USEPOOLIFNODESKTOP, OCF_USEPOOLQUOTA
       0 TYPE_MENU                     0xfffff960001515ac OCF_DESKTOPHEAP, 
OCF_PROCESSOWNED
       0 TYPE_CURSOR          Uscu     0xfffff960001541a0 OCF_MARKPROCESS, 
OCF_PROCESSOWNED, OCF_USEPOOLQUOTA
       0 TYPE_SETWINDOWPOS    Ussw     0xfffff960001192b4 OCF_THREADOWNED, 
OCF_USEPOOLQUOTA
       0 TYPE_HOOK                     0xfffff9600018e5c8 OCF_DESKTOPHEAP, 
OCF_THREADOWNED
       0 TYPE_CLIPDATA        Uscb     0xfffff9600017c5ac 
[snip]
       0 TYPE_WINEVENTHOOK    Uswe     0xfffff9600018f148 OCF_THREADOWNED
       0 TYPE_TIMER           Ustm     0xfffff960001046dc OCF_PROCESSOWNED
       0 TYPE_INPUTCONTEXT    Usim     0xfffff9600014c660 OCF_DESKTOPHEAP, 
OCF_THREADOWNED
       0 TYPE_HIDDATA         Usha     0xfffff960001d2a34 OCF_THREADOWNED
       0 TYPE_DEVICEINFO      UsDI     0xfffff960000d8cd4 
       0 TYPE_TOUCH           Ustz     0xfffff9600017c5cc OCF_THREADOWNED
       0 TYPE_GESTURE         Usgi     0xfffff9600017c5cc OCF_THREADOWNED
```

The `TYPE_WINDOW` objects are thread-owned; they are allocated from the desktop heap (or failing that, the session pool), and the individual allocations are tagged with the “`Usdw`” bytes.

`TYPE_TIMER` objects are process-owned. `TYPE_CLIPDATA` objects are neither thread- nor process-owned, which makes sense because data copied to the clipboard is freely accessible by any process in the session that calls `GetClipboardData`.

### The Shared Info Structure

Although you can indeed use the previously mentioned `dwAllocTag` to locate USER objects in RAM, there is a more authoritative method that has less potential to yield false positives. In particular, the `win32k!_gSharedInfo` symbol points to a `tagSHAREDINFO` structure, which in turn identifies the location of the session’s USER handle table—a map to all USER objects in use on the system. By finding objects through their references in the handle table, you know they are (or recently were) USER objects, in contrast with simply scanning through RAM looking for the 4-byte tags.

Finding the `win32k!_gSharedInfo` symbol accurately and reliably across all Windows versions can be difficult, especially without the use of PDB files. One thing you know, however, is that the symbol is somewhere in the `win32k.sys` kernel module. Already that clue narrows your search down to 3–5MB. Then do a little digging; open `win32k.sys` in IDA Pro and use the Names pane to locate `_gSharedInfo`. As shown in [Figure 15-4](#figure15-4), the symbol exists in the data section of the PE file. Depending on the build of the `win32k.sys` you’re analyzing, you can now narrow the search to about 100–150KB.

At this point, you can use some basic pattern matching to find the structure. Yet before you do that, you need to know a little bit more about the values you are trying to match. On a Windows 7 x64 system, the `tagSHAREDINFO` looks like this:

```
>>> dt("tagSHAREDINFO")
'tagSHAREDINFO' (568 bytes)
0x0   : psi                            ['pointer64', ['tagSERVERINFO']]
0x8   : aheList                        ['pointer64', ['_HANDLEENTRY']]
0x10  : HeEntrySize                    ['unsigned long']
0x18  : pDispInfo                      ['pointer64', ['tagDISPLAYINFO']]
0x20  : ulSharedDelta                  ['unsigned long long']
0x28  : awmControl                     ['array', 31, ['_WNDMSG']]
0x218 : DefWindowMsgs                  ['_WNDMSG']
0x228 : DefWindowSpecMsgs              ['_WNDMSG']
```

![c15f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f004.png)

*[**Figure 15-4:**](#figureanchor15-4) Analyzing win32k.sys in IDA Pro for clues on how to find the shared information structure*

---

## **Key Points**

The key points are these:

- `psi`: Points to a valid `tagSERVERINFO` structure.
- `aheList`: Points to an array of `_HANDLEENTRY` structures—one for each handle in the table. To determine the number of handles currently in use, you can look at `tagSHAREDINFO.psi.cHandleEntries`.
- `HeEntrySize`: The size of a `_HANDLEENTRY` for the current platform.
- `ulSharedDelta`: A delta that user-mode processes can use to determine the location of USER objects in kernel memory.

---

### Algorithm for Finding Shared Info

With the information you have gathered so far, it is possible to write code for Volatility that can, in fact, find this needle in a haystack. The procedure is as follows:

1. Determine the base address of `win32k.sys`as mapped into the session space.
2. Locate the data PE section. If the PE header is corrupt or paged (we have seen this happen in very busy, large memory systems), fall back to brute forcing the search using the 3–5MB full length of `win32k.sys` instead of just the data section.
3. Iterate over the data on a 4-byte boundary and instantiate a `tagSHAREDINFO` object at each address; then call the object’s `is_valid` method to perform the necessary sanity checks.

To see the Python code that corresponds to the described steps, look in the `volatility/plugins/gui/win32k_core.py` file. Now you should be able to find the `tagSHAREDINFO` structure on all versions of Windows without any problem.

---

**NOTE**

Tarjei Mandt’s *Windows Hook of Death: Kernel Attacks through Usermode Callbacks* ([http://mista.nu/blog/2011/08/11/windows-hooks-of-death-kernel-attacks-through-user-mode-callbacks/](http://mista.nu/blog/2011/08/11/windows-hooks-of-death-kernel-attacks-through-user-mode-callbacks/)) describes two additional ways of finding `tagSHAREDINFO`, which you might also find useful, depending on your context. For example, you can use `user32!_gSharedInfo` (an exported symbol available on Windows 7); or on a live system, you can call the `CsrClientConnectToServer` API function.

---

### Handle Table Entries

A handle table entry on Windows 7 x64 looks like this:

```
>>> dt("_HANDLEENTRY")
'_HANDLEENTRY' (24 bytes)
0x0   : phead                          ['pointer64', ['_HEAD']]
0x8   : pOwner                         ['pointer64', ['void']]
0x10  : bType                          ['Enumeration', {'target': 
  'unsigned char', 'choices': {0: 'TYPE_FREE', 1: 'TYPE_WINDOW', 
  2: 'TYPE_MENU', 3: 'TYPE_CURSOR', 4: 'TYPE_SETWINDOWPOS', 5: [snip]
0x11  : bFlags                         ['unsigned char']
0x12  : wUniq                          ['unsigned short']
```

All USER objects start with one of the common headers, which are pointed to by the `_HANDLEENTRY.phead` member. The `bType` tells you what type of object the handle is for, and based on the information previously dumped from `win32k!_gahti`, you know which objects are thread-owned and which are process-owned. The ones owned by a thread begin with `_THRDESKHEAD`, and those owned by a process begin with `_PROCDESKHEAD`. Objects such as `TYPE_CLIPBOARD` begin with the generic `_HEAD`. Here are the three possibilities:

```
>>> dt("_HEAD")
'_HEAD' (16 bytes)
0x0   : h                              ['pointer64', ['void']]
0x8   : cLockObj                       ['unsigned long']
>>> dt("_THRDESKHEAD")
'_THRDESKHEAD' (40 bytes)
0x0   : h                              ['pointer64', ['void']]
0x8   : cLockObj                       ['unsigned long']
0x10  : pti                            ['pointer64', ['tagTHREADINFO']]
0x18  : rpdesk                         ['pointer64', ['tagDESKTOP']]
0x20  : pSelf                          ['pointer64', ['unsigned char']]
>>> dt("_PROCDESKHEAD")
'_PROCDESKHEAD' (40 bytes)
0x0   : h                              ['pointer64', ['void']]
0x8   : cLockObj                       ['unsigned long']
0x10  : hTaskWow                       ['unsigned long']
0x18  : rpdesk                         ['pointer64', ['tagDESKTOP']]
0x20  : pSelf                          ['pointer64', ['unsigned char']]
```

If an object in the handle table is thread-owned, you can identify the object’s specific owning thread by referencing `_THRDESKHEAD.pti.pEThread`, which points to the executive `_ETHREAD` structure. On the other hand, if an object is process-owned, the `_HANDLEENTRY.pOwner` field is a pointer to the owning `tagPROCESSINFO`. From there, `tagPROCESSINFO.Process` identifies the executive `_EPROCESS` structure.

Now, regardless of the situation, you can always track USER objects back to their owning thread or process.

### Enumerating a Session’s USER Handles

The `userhandles` plugin locates the shared information structure for each session, walks the handle table, and prints out the contents:

```
$ python vol.py -f win7x64.dd --profile=Win7SP1x64 userhandles
Volatility Foundation Volatility Framework 2.4
**************************************************
SharedInfo: 0xffffc96c00f005d300, SessionId: 0 
aheList: 0xfffff900c0400000, Table size: 0x2000, Entry size: 0x18

Object(V)                Handle bType          Flags    Thread  Process
------------------ ------------ --------------- -------- -------- -------
0xfffff900c05824b0      0x10001 TYPE_MONITOR     0     -------- -
0xfffff900c01bad20      0x10002 TYPE_WINDOW      64      432    316
0xfffff900c00b6730      0x10003 TYPE_CURSOR      0     -------- 316
0xfffff900c0390b90      0x10004 TYPE_WINDOW      0       432    316
0xfffff900c00d7ab0      0x10005 TYPE_CURSOR      0     -------- 316
0xfffff900c0390e60      0x10006 TYPE_WINDOW      0       432    316
0xfffff900c00d7640      0x10007 TYPE_CURSOR      0     -------- 316
[snip]
0xfffff900c0630bf0   0x467c054b TYPE_HOOK        0       2368   2348
0xfffff900c0616d60     0x72055f TYPE_MENU        0     -------- 880
0xfffff900c0654610   0x494c0581 TYPE_MENU        0     -------- 880
0xfffff900c1a14b10   0x539c05f083 TYPE_CURSOR      0     -------- 880
[snip]
```

This plugin has three additional command-line arguments:

```
$ python vol.py userhandles –-help
[snip]
  -p PID, --pid=PID     Pid filter
  -t TYPE, --type=TYPE  Handle type
  -F, --free            Include free handles
```

To show only USER objects owned by a particular process, use the `--pid=PID` option. To filter by object type, use something similar to the following: `--type=TYPE_HOOK`. Finally, if you want to include information on handles marked as freed, use `--free`. You may be interested in freed objects if you are searching for evidence of events that occurred in the recent past.

Although the output of this plugin is not very verbose, it gives you an overview of the types of objects that a particular thread or process uses. Furthermore, you can leverage it as an API for other plugins, such as `eventhooks` and `clipboard`, which are discussed next.

## Event Hooks

Applications can use event hooks to receive notification when certain UI-related events occur. For example, Windows Explorer fires events when sounds play (`EVENT_SYSTEM_SOUND`); when a scroll operation begins (`EVENT_SYSTEM_SCROLLSTART`); or when an item in a menu bar, such as the Start menu, is selected (`EVENT_SYSTEM_MENUSTART`). If a client application wants to display a small speaker icon in the system tray when a sound is emitted, they can synchronize the behavior by installing an event hook that listens for `EVENT_SYSTEM_SOUND`.

Similar to message hooks (see Chapter 14), you can use event hooks to generically load a DLL into any processes that fire events, such as `explorer.exe`. This is a quick and effective way to execute code in the context of a remote process. The low-level internals and data structures are undocumented, which explains why you don’t have many tools, much less forensic tools, to analyze installed event hooks.

---

## **Analysis Objectives**

Your objectives are these:

- **Determine event hook scope**: You can inspect RAM for artifacts created during event hook installation. They will tell you which processes and threads are affected, as well as the specific events being monitored.
- **Analyze intent**: By disassembling the event hook function code, you can figure out exactly why the hooks were being used.

## **Data Structures**

The data structure for event hooks is `tagEVENTHOOK`. Microsoft does not document this internal structure, so the fields and offsets were determined through reverse engineering.

```
>>> dt("tagEVENTHOOK")
'tagEVENTHOOK' (None bytes)
0x18  : phkNext             ['pointer', ['tagEVENTHOOK']]
0x20  : eventMin            ['Enumeration', {'target': 'unsigned long', 
  'choices': {1: 'EVENT_MIN', 2: 'EVENT_SYSTEM_ALERT', [snip]
0x24  : eventMax            ['Enumeration', {'target': 'unsigned long', 
  'choices': {1: 'EVENT_MIN', 2: 'EVENT_SYSTEM_ALERT', [snip]
0x28  : dwFlags             ['unsigned long']
0x2c  : idProcess           ['unsigned long']
0x30  : idThread            ['unsigned long']
0x40  : offPfn              ['unsigned long long']
0x48  : ihmod               ['long']
```

Event hooks are installed by calling `SetWinEventHook`. As you can see from the function prototype that follows, most of the parameters are named the same as the underlying data structures in kernel mode.

```
HWINEVENTHOOK WINAPI SetWinEventHook(
  _In_  UINT eventMin,
  _In_  UINT eventMax,
  _In_  HMODULE hmodWinEventProc,
  _In_  WINEVENTPROC lpfnWinEventProc,
  _In_  DWORD idProcess,
  _In_  DWORD idThread,
  _In_  UINT dwflags
);
```

To hook all event types, applications can specify `EVENT_MIN` and `EVENT_MAX` as the `eventMin` and `eventMax` parameters, respectively. Malware that leverages event hooks as simply a means to inject DLLs into processes often use this pairing because they do not care which specific events are being generated.

## **Key Points**

The key points are these:

- `phkNext`: The next hook in the chain.
- `eventMin`: The lowest system event that the hook applies to.
- `eventMax`: The highest system event that the hook applies to.
- `dwFlags`: Tells you if the process generating the event will load the DLL containing the event hook procedure into its address space (`WINEVENT_INCONTEXT`). It also tells you if the thread installing the hook wants to be exempt from the hook (`WINEVENT_SKIPOWNPROCESS` and `WINEVENT_SKIPOWNTHREAD`).
- `idProcess`: The process ID (PID) of the target process, or 0 for all processes in the desktop.
- `idThread`: The thread ID (TID) of the target thread, or 0 for all threads in the desktop.
- `offPfn`: An RVA to the hook procedure in the DLL.
- `ihmod`: An index into the `win32k!_aatomSysLoaded` array, which you can use to identify the full path to the DLL containing the hook procedure.

---

Now you can view the full `eventhooks` output for this Windows 7 x64 system:

```
$ python vol.py -f  win7x64.dd --profile=Win7SP1x64 eventhooks
Volatility Foundation Volatility Framework 2.4

Handle: 0x300cb, Object: 0xfffff900c01eda10, Session: 1
Type: TYPE_WINEVENTHOOK, Flags: 0, Thread: 1516, Process: 880
eventMin: 0x4 EVENT_SYSTEM_MENUSTART
eventMax: 0x7 EVENT_SYSTEM_MENUPOPUPEND
Flags: , offPfn: 0xff567cc4, idProcess: 0, idThread: 0
ihmod: -1
```

One event hook is installed by a thread of `explorer.exe` PID 880. The types of events being filtered include menu start and stop operations. Because `ihmod` is -1 for the hook, you know that the `offPfn` hook procedure is located in `explorer.exe`, not in an external DLL. So this hook is probably not malicious. If the hooks were malicious, it would look very similar to the message hooks described earlier in the chapter: It would be global, and the hook procedure would be inside an attacker-provided DLL.

## Windows Clipboard

Determining what’s in a computer’s clipboard can be a valuable resource. For example, in one scenario, we traced a Remote Desktop Protocol (RDP) user’s actions by dumping his command history and seeing him start an outgoing FTP transaction from the victim computer. We could see the FTP server address and the user’s login name, but not the password. In this case, the attacker copied the password to his clipboard and pasted it over the RDP channel. Using both the command history and clipboard extraction plugins, we recovered the full set of credentials from RAM.

---

## **Analysis Objectives**

Your objectives are these:

- **Password recovery**: You can extract the contents of clipboard data from RAM, which in some cases can contain sensitive information such as passwords, usernames, and so on.
- **Copied files artifacts**: Attackers often exfiltrate files from victim systems to their remote drop sites. They can accomplish this by using basic copy and paste from Windows Explorer into an FTP directory—in which case the full path to the source file copies into the clipboard.

## **Data Structures**

The two critical structures for understanding clipboard objects are `tagCLIP` and `tagCLIPDATA`. The `tagCLIP` structure is defined in Windows 7 PDB files; however, `tagCLIPDATA` is not divulged at all.

In the earlier section on window stations, you learned that `tagWINDOWSTATION.pClipBase` points to an array of `tagCLIP` structures. The `tagCLIP` specifies the clipboard format and contains a handle to an associated `tagCLIPDATA`. You must separately obtain the actual address of the `tagCLIPDATA` object and then match it with the handle value. The easiest way to locate all `tagCLIPDATA` objects is to walk the session handle tables and filter for `TYPE_CLIPDATA`.

Here is how the structures appear on Windows 7 x64:

```
>>> dt("tagCLIP")
'tagCLIP' (24 bytes)
0x0   : fmt                            ['Enumeration', {'target': 'unsigned long', 'choices': {128: 'CF_OWNERDISPLAY', 1: 'CF_TEXT', 2: 'CF_BITMAP', 3: 'CF_METAFILEPICT', 4: 'CF_SYLK', 5: 'CF_DIF', 6: 'CF_TIFF', 7: 'CF_OEMTEXT', 8: 'CF_DIB', 9: 'CF_PALETTE', 10: 'CF_PENDATA', 11: 'CF_RIFF', 12: 'CF_WAVE', 13: 'CF_UNICODETEXT', 14: 'CF_ENHMETAFILE', 15: 'CF_HDROP', 16: 'CF_LOCALE', 17: 'CF_DIBV5', 131: 'CF_DSPMETAFILEPICT', 129: 'CF_DSPTEXT', 130: 'CF_DSPBITMAP', 142: 'CF_DSPENHMETAFILE'}}]
0x8   : hData                          ['pointer64', ['void']]
0x10  : fGlobalHandle                  ['long']

>>> dt("tagCLIPDATA")
’tagCLIPDATA’ (None bytes)
0x10  : cbData                         ['unsigned int']
0x14  : abData                         ['array', <function <lambda> at 0x1048c55e000>, ['unsigned char']]
```

## **Key Points**

The key points are these:

- `fmt`: Specifies the clipboard format. Although the enumeration includes only standard formats, applications can create their own with `RegisterClipboardFormat`. You can only expect formats with “TEXT” in the name to contain printable characters.
- `hData`: A handle value for the associated `tagCLIPDATA` object. This value can also be 1 for `DUMMY_TEXT_HANDLE`, 2 for `DUMMY_DIB_HANDLE`, or 0 for certain deferred operations, as described in *How the Clipboard Works* ([http://blogs.msdn.com/b/ntdebugging/archive/2012/03/16/how-the-clipboard-works-part-1.aspx](http://blogs.msdn.com/b/ntdebugging/archive/2012/03/16/how-the-clipboard-works-part-1.aspx)).
- `abData`: An array of bytes (length `cbData`) that contains the actual clipboard data. It can be text or binary, depending on the format.

---

### Algorithm for Clipboard Extraction

At the Digital Forensics Research Conference (DFRWS) 2011, Okolica and Peterson ([http://www.dfrws.org/2011/proceedings/18-350.pdf](http://www.dfrws.org/2011/proceedings/18-350.pdf)) were first to present a technique for extracting clipboard contents from RAM using a tool called the Compiled Memory Analysis Tool (CMAT). They discussed methods to find the data from user- and kernel-mode by using PDB files from Microsoft’s symbol server to resolve `user32!gphn` and `win32k!gSharedInfo`, respectively.

The way Volatility’s plugin works is similar yet quite different at the same time. Here is a brief description of the steps:

1. Enumerate all unique `_MM_SESSION_SPACE` structures.
2. Find the `tagSHAREDINFO` for each session and walk the USER handle table, collecting all `TYPE_CLIPDATA` objects.
3. Scan RAM for window station objects and enumerate the `tagCLIP` structures from `tagWINDOWSTATION.pClipBase`.
4. Associate the `tagCLIP.hData` handle values with their corresponding `tagCLIPDATA`.
5. At the end, cycle through any remaining `tagCLIPDATA` objects found via the USER handle table that were not already associated with a `tagCLIP`. This allows you to still report clipboard data even if the window station object is not found.

### Recovering Text from the Clipboard

One of the publicly accessible memory images known to have data in the clipboard at the time of the acquisition is `dfrws2008-rodeo-memory.img`. In the code that follows, you can see the output of the plugin on this image:

```
$ python vol.py -f dfrws2008-rodeo-memory.img --profile=WinXPSP2x86 clipboard
Volatility Foundation Volatility Framework 2.4
Session WindowStation Format               Handle Object     Data
------- ------------- ---------------- ---------- ---------- ------------
   0    WinSta0       CF_UNICODETEXT     0x4900c3 0xe12a7c98 pp -B -p -o out.pl file 
   0    WinSta0       CF_LOCALE           0x80043 0xe12362d0     
   0    WinSta0       CF_TEXT                 0x1 ----------     
   0    WinSta0       CF_OEMTEXT              0x1 ----------
```

As you can see, a user in session `0\WinSta0` placed a Unicode string `pp –B –p –o out.pl file` into the clipboard. This seems to be part of a command that runs a Perl script.

The `CF_LOCALE` format is not shown because it is binary, but you can view a hex dump by passing `-v/--verbose` to the plugin. There is also no data shown for `CF_TEXT` or `CF_OEMTEXT` because the handle value is 1 (`DUMMY_TEXT_HANDLE`).

### Recovering Binary Data from the Clipboard

In the following example, Microsoft Word and Microsoft Paint were opened. A small sketch was created in Paint, copied to the clipboard, and then pasted into Word. You can see the various private Object Linking and Embedding (OLE) formats created by Word (see the previously referenced paper by Okolica and Peterson for more information on private formats). Additionally, there are new `CF_METAFILEPICT`, `CF_ENHMETAFILE`, `CF_BITMAP`, and `CF_DIBV5` formats to support the binary images in the clipboard. Data is not shown because it is binary, but with a small amount of work, the images could be carved from memory and saved for viewing on an analysis system.

---

**NOTE**

You have alternate ways to recover images from RAM, including carving over the physical file with tools such as foremost or scalpel, or using the `dumpfiles` Volatility plugin. Here, the point is to identify specific images as the ones in the clipboard.

---

```
$ python vol.py -f image_clip.vmem --profile=Win7SP1x86 clipboard 
Volatility Foundation Volatility Framework 2.4
Session  WindowStation Format                 Handle Object     
-------- ------------- ------------------ ---------- ---------- 
     1   WinSta0       0xc009              0x3a2043b 0xfd91a160 
     1   WinSta0       0xc00b                    0x0 ---------- 
     1   WinSta0       0xc004                    0x0 ---------- 
     1   WinSta0       0xc003                    0x0 ---------- 
     1   WinSta0       0xc00e                    0x0 ---------- 
     1   WinSta0       CF_METAFILEPICT           0x0 ---------- 
     1   WinSta0       CF_DIB              0x20d04cf 0xfe1a0000 
     1   WinSta0       0xc013               0xb202c7 0xfe4d9650 
     1   WinSta0       CF_ENHMETAFILE            0x3 ---------- 
     1   WinSta0       CF_BITMAP          0xc3050d23 ---------- 
     1   WinSta0       CF_DIBV5                  0x2 ----------
```

In the next example, a user selected a file on the desktop and pressed Ctrl+C to copy it to another directory. As you might suspect, the entire file contents is not copied to the clipboard in this case. Instead, an object of the `CF_HDROP` format is created with a full path to the file to be copied:

```
$ python vol.py -f xpsp3.vmem --profile=WinXPSP3x86 clipboard -v
Volatility Foundation Volatility Framework 2.4
 [snip]

   0    WinSta0       CF_HDROP           0x10230131 0xe1fa6590

0xe1fa659c  14 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 ................
0xe1fa65ac  01 00 00 00 43 00 3a 00 5c 00 44 00 6f 00 63 00 ....C.:.\.D.o.c.
0xe1fa65bc  75 00 6d 00 65 00 6e 00 74 00 73 00 20 00 61 00 u.m.e.n.t.s...a.
0xe1fa65cc  6e 00 64 00 20 00 53 00 65 00 74 00 74 00 69 00 n.d...S.e.t.t.i.
0xe1fa65dc  6e 00 67 00 73 00 5c 00 41 00 64 00 6d 00 69 00 n.g.s.\.A.d.m.i.
0xe1fa65ec  6e 00 69 00 73 00 74 00 72 00 61 00 74 00 6f 00 n.i.s.t.r.a.t.o.
0xe1fa65fc  72 00 5c 00 44 00 65 00 73 00 6b 00 74 00 6f 00 r.\.D.e.s.k.t.o.
0xe1fa660c  70 00 5c 00 6e 00 6f 00 74 00 65 00 2e 00 74 00 p.\.n.o.t.e...t.
0xe1fa661c  78 00 74 00 00 00 00 00                         x.t.....
```

Keep in mind that with the `clipboard` plugin, you are recovering clipboard data from all sessions and all window stations. That means if multiple users are logged on (one at the console, one via RDP, etc.), you can extract everyone’s clipboard data.

## Case Study: ACCDFISA Ransomware

The Anti Cyber Crime Department of Federal Internet Security Agency (ACCDFISA) malware, as described by Emsisoft ([http://blog.emsisoft.com/2012/04/11/the-accdfisa-malware-family-ransomware-targetting-windows-servers/](http://blog.emsisoft.com/2012/04/11/the-accdfisa-malware-family-ransomware-targetting-windows-servers/)), is a ransomware that creates a new desktop to display the ransom notice and effectively locks users out of the system until they enter a special code. For example, one of the variants displays the message shown in [Figure 15-5](#figure15-5) after an infected system boots:

![c15f005.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f005.png)

*[**Figure 15-5:**](#figureanchor15-5) The malware’s ransomware desktop message*

With no obvious way to get back to the real desktop, users are forced to comply with the attacker’s demands or figure out some other way around it. As shown in [Figure 15-6](#figure15-6), to create this screen-lock effect, the malware uses `CreateDesktopA` to create a new desktop named `My Desktop 2` and then switches to it with `SwitchDesktop`.

![c15f006.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f006.png)

*[**Figure 15-6:**](#figureanchor15-6) Code disassembly of the malware preparing the new desktop*

The artifacts this malware leaves in physical memory should not be surprising: a suspiciously named desktop with a single process (besides the typical `csrss.exe`) associated with the desktop. In the output that follows, notice that the desktop is `WinSta0\My Desktop 2`, and the only thread attached to the desktop (besides those from `csrss.exe`) is thread ID 308 from `svchost.exe`. As you can imagine, a single thread running alone in a desktop is not typical.

```
$ python vol.py –f ACCFISA.vmem --profile=WinXPSP3x86 deskscan
Volatility Foundation Volatility Framework 2.4
[snip]
**************************************************
Desktop: 0x24675c0, Name: WinSta0\My Desktop 2, Next: 0x820a47d8
SessionId: 0, DesktopInfo: 0xbc310650, fsHooks: 0
spwnd: 0xbc3106e8, Windows: 111
Heap: 0xbc310000, Size: 0x300000, Base: 0xbc310000, Limit: 0xbc610000
 652 (csrss.exe 612 parent 564)
 648 (csrss.exe 612 parent 564)
 308 (svchost.exe 300 parent 240)
```

Let’s view a tree of the windows in the suspicious desktop. So far, you know that `svchost.exe` is most likely the malware process, but you need a bit more evidence to explicitly link it with the ransomware message.

```
$ python vol.py -f ACCFISA.vmem --profile=WinXPSP3x86 wintree
Volatility Foundation Volatility Framework 2.4
 [snip]
**************************************************
Window context: 0\WinSta0\My Desktop 2
[snip]
.#100e2  csrss.exe:612 -
.#100e4  csrss.exe:612 -
#100de (visible) csrss.exe:612 -
.Anti-Child Porn Spam Protection (18 U.S.C. ? 2252) (visible) svchost.exe:300 WindowClass_0
..Send Code (visible) svchost.exe:300 Button
..#100ee (visible) svchost.exe:300 Edit
..Your Id #: 1074470467 Our special service email: security11220@gmail.com (visible) svchost.exe:300 Static
..Your ID Number and our contacts (please write down this data): (visible) svchost.exe:300 Static
..#100e8 (visible) svchost.exe:300 Static
[snip]
```

As you can see, all the windows for the ransom message are owned by `svchost.exe` with process ID 300. Now you can start your initial investigation based on this specific process. For example, using `dlllist` shows that it is not a real `svchost.exe` because it is hosted out of the `C:\wnhsmlud` directory:

```
$ python vol.py -f ACCFISA.vmem --profile=WinXPSP3x86 dlllist -p 300
Volatility Foundation Volatility Framework 2.4
************************************************************************
svchost.exe pid:    300
Command line : "C:\wnhsmlud\svchost.exe" 
Service Pack 3

Base             Size Path
---------- ---------- ----
0x00400000    0x2f000 C:\wnhsmlud\svchost.exe
0x7c900000    0xb2000 C:\WINDOWS\system32\ntdll.dll
0x7c800000    0xc60f000 C:\WINDOWS\system32\kernel32.dll
0x77c10000    0x58000 C:\WINDOWS\system32\MSVCRT.dll
[snip]
```

With information on the executable path name, you can search for it in the registry. In the following output, it is easily locatable in the registry hives cached in memory:

```
$ python vol.py -f ACCFISA.vmem --profile=WinXPSP3x86 printkey 
      -K "Microsoft\Windows\CurrentVersion\Run"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\software
Key name: Run (S)
Last updated: 2012-07-23 01:57:05 
Subkeys:

Values:
REG_SZ        VMware Tools    : (S) "C:\Program Files\VMware\VMware 
Tools\VMwareTray.exe"
REG_SZ        VMware User Process : (S) "C:\Program Files\VMware\VMware 
Tools\VMwareUser.exe"
REG_SZ        SunJavaUpdateSched : (S) "C:\Program Files\Common Files\
Java\Java Update\jusched.exe"
REG_SZ        svchost         : (S) C:\wnhsmlud\svchost.exe
```

You followed the artifacts in the GUI subsystem and they led you straight to the malware’s running process and its persistence mechanism.

## Summary

Analysts and investigators often overlook hooks in the Windows GUI subsystem. In fact, the messaging and event-dispatching architecture is too frequently ignored altogether. Now that you’ve seen examples of detecting malware in memory by analyzing the related artifacts, you can begin to integrate such checks into your cases. Furthermore, data in a user’s clipboard, the names of desktops objects, and the messages displayed in window titles are also valuable sources of evidence.
