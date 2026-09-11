# Chapter 17 Event Reconstruction

Reconstructing an event is a necessary step in most forensics investigations. Although you could probably pick any chapter in this book and say it facilitates correlations, triage, and so forth, extracting strings and recovering attacker command histories are two procedures that stand out as notably significant. Despite the fact that extracting strings is one of the most ancient forms of analysis, it’s still extremely powerful, especially when combined with the capability to add context (such as linking the strings with their owning process or kernel module).

This chapter shows you several ways to leverage strings to prove or disprove that certain actions took place on a system. You’ll also learn about the internals of the Windows command architecture that attackers frequently exploit to navigate the breached network, install or configure backdoors, mount shares, and so on. For example, if you use `cmd.exe` as an FTP client, you might find evidence that identifies the server, the attacker’s username and password, and the FTP commands—long after the actual network connections are torn down.

## Strings

As introduced in Chapter 2, a string is a sequence of bytes that contains human-readable characters. Although strings can exist in various encodings, the most common ones you’ll analyze are ASCII and Unicode. They are the encodings in which the Windows application programming interfaces (APIs) expect to receive their arguments. For example, `CreateFileA ...`
