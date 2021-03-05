# stopper

this is an effort at an async concurrency primitive to stop both streams and futures. this was created to facilitate graceful shutdown in a http framework, but likely has utility beyond that.

warning: this may have race conditions and has not been thoroughly vetted or used yet. it currently exists as a functional sketch of the interface i needed. if you use this, please file bug reports. i want this to be rock solid in the near future.
