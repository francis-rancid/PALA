# Chapter 24 File Systems in Memory

As files are opened, created, read, and written, the operating system caches information about these actions in a number of data structures. The associated artifacts include the directory structure, metadata (including timestamps), and even the contents of recently accessed files. Particularly on Linux, in which memory-only file systems are used on nearly every distribution, such artifacts are lost when the machine is powered down. Thus, in many cases, preserving RAM is the best (and sometimes the only) method to determine which files an attacker accesses, where a rootkit hides, or what was introduced as the result of a client-side browser attack.

## Mounted File Systems

Linux maintains a list of the actively mounted file systems in kernel memory. One of the most basic analysis tasks is to locate this list and get an initial impression of which file systems were accessible. The direction of your investigation can be affected based on whether a file was opened from the local hard disk, over a remote Network File System (NFS) or Server Message Block (SMB) drive, or an external USB stick.

## **Analysis Objectives**

Your objectives are these:

- **Identify reconnaissance and snooping:** Many companies have internal file servers hosting intellectual property. Any semicompetent attacker will immediately try to find and mount interesting network shares, to either grab specific files or search for any files containing certain terms relevant to the attacker’s motives. ...
