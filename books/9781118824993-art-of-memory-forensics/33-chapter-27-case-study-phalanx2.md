# Chapter 27 Case Study: Phalanx2

Phalanx2 (P2) is a sophisticated Linux kernel rootkit discovered during a number of high-profile incidents involving some of the world’s most sensitive networks. Not only does P2 try to hide from common system administration tools running on live systems, but it also includes capabilities to frustrate reverse engineering and memory forensics. Of the Linux kernel rootkits that have been discussed publicly, P2 is by far the most advanced one that we have seen and analyzed.

This chapter takes you through a deep analysis of a number of interesting components of P2. This analysis also demonstrates how memory forensics can be combined with static analysis, dynamic reverse engineering, and baseline comparison techniques to analyze even the most sophisticated Linux rootkits.

## Phalanx2

P2 is often considered an infamous Linux kernel rootkit due to the number of high-profile investigations in which it was encountered. It is also very difficult to detect using common system administration and live forensics tools. To accomplish this, P2 employs a mix of function pointer overwrites and system call hooks to hide its files, processes, and network connections. As a result, nearly all host monitoring and integrity checking systems cannot identify compromised systems.

The source code of the original version of Phalanx was leaked to Packetstorm in 2005 ([http://packetstormsecurity.com/files/42556/phalanx-b6.tar.bz2.html](http://packetstormsecurity.com/files/42556/phalanx-b6.tar.bz2.html)). Since that time, there have been no further ...
