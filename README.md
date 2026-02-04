# rust_frib_datasource

This repository provides a rust module that support source independent
access of FRIB/NSCLDAQ ring item data.  Ring items can come from live
(ring buffers) or they can come from offline (file) data.

Data sources are specified by URIs of the form:

Data sources and data sinks can be created. 

See the crate documentation online pointed to at 
https://crates.io/crates/frib_datasource


sourcetype://source-specification

Valid source/sink types are:

* ```tcp``` data comes from a (possibly) remote ring item.
* ```file``` data comes from a file.

## Online (tcp) source/sink-specification

These have two components.  The first is the host on which the ring item is
located.  The second is the name of the ringbuffer in that host.  For example:

```
tcp://spdaq08.frib.msu.edu/aring
```

Is a data source from the ring named ```aring``` in the host
```spdaq08.frib.msu.edu```


Note that to use online ring buffers, the FRIB/NSCLDAQ port manager and
ringmaster must be running in all involved systems.  In the example URI above,
the ringmaster and port manager must be running in both the requesting host and
spdaq08.frib.msu.edu

If specifying a sink, the host _must_ be ```localhost```

## Offline (file) source-specification

This is just the path to the file that contains the data.  For example:

```
file://./run-0001-01.evt
```

Specifies ```run-0001-01.evt``` in the current working directory.
Note that:
*  ~ expansion is not supported
* Environment variable name substitution is not supported.
* The special value ```-``` represents stdin.  This is useful to allow
a program to operate as a pipeline element.


