# Chapter 7 Process Memory Internals

Even after years of experience digging through RAM, the amount of evidence you can find in process memory never ceases to amaze us. Although artifacts in kernel memory, such as an `_EPROCESS`, can provide useful details, the actual memory that the process uses is highly concentrated with data that can reveal valuable information about the process’ current state. This data includes, but is not limited to, all content processed by the application (received via the network or interactive user input); mapped files; shared libraries; passwords; credit card transactions (for point-of-sale [POS] systems); and private structures for e-mail, documents, and chat logs.

This chapter analyzes the various application programming interfaces (APIs) used for allocating the different data types and examines how to enumerate process memory regions via memory forensics. In doing so, you’ll see how to leverage characteristics (such as permissions, flags, and sizes) of memory ranges to deduce what kind of data they contain. You’ll also learn the tools and techniques for extracting all process memory (at least what’s addressable and memory-resident) or individual ranges to a file, which enables you to further analyze it with external tools such as virus scanners, disassemblers, and so on. Finally, we present several ways to search for patterns within process memory, involving both Volatility’s APIs and Yara signatures.

## What’s in Process Memory?

[Figure 7-1](#figure7-1) shows a ...
