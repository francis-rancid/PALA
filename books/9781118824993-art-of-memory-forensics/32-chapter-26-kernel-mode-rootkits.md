# Chapter 26 Kernel Mode Rootkits

Kernel mode rootkits are extremely dangerous to the runtime integrity of a Linux system. These rootkits have the power to add, delete, or modify any data that kernel or userland applications request about the state of the system. This can include information such as lists of running processes, loaded kernel modules, active network connections, files within a directory, and even the contents of those files. Kernel mode rootkits can also monitor user activity including keystrokes, network packets, and interactions with hardware such as removable media or security devices. To find kernel mode rootkits, you must perform deep inspection of the running kernel, including its code and data structures.

Kernel rootkits can hook many places to subvert the system, and accordingly, Volatility has powerful support for enumerating and verifying in-kernel data. Many of these are newly developed capabilities and exclusive to Volatility, such as finding Netfilter hooks and copied credential structures. Throughout this chapter, you’ll see several of these plugins in action and you’ll learn how to use them in your own investigations.

## Accessing Kernel Mode

To install and use a kernel mode rootkit, attackers must first obtain root-level privileges. They can gain access to this permission level either through a social engineering attack (e.g., convincing an administrator to install a malicious application) or by remotely exploiting a network service that runs with ...
