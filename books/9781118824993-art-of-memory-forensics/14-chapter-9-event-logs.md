# Chapter 9 Event Logs

Event logs contain a wealth of forensic information and are a staple in almost any type of investigation. They contain details about application errors (such as when Word crashes after a heap-spray exploit), interactive and remote logins, changes in the firewall policy, and other events that have occurred on the system. Combined with the timestamps that are supplied with each event, the logs can help you determine exactly what happened on a system, or at least give you a timeframe on which to focus the rest of your efforts.

This chapter covers how to locate event logs in RAM and parse them for forensic purposes. Many of the log files are mapped into memory during the run time of the system, so it is typical to find hundreds, if not thousands, of individual records in memory dumps. In some cases, you may even be able to extract entries after they are marked for deletion by an administrator or maliciously cleared by an attacker.

## Event Logs in Memory

Because event records are recorded throughout the run time of a system, it makes sense that you will find these records, or even the event logs files, in memory. To find records or event logs, you first need to know their structure—what they look like and where to find them in a consistent manner—because methodologies vary greatly depending on the target operating system.

## **Analysis Objectives**

Your objectives are these:

- **Locate event logs in memory:** Starting with Vista, Microsoft made several critical changes ...
