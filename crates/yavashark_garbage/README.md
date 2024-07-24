The `yavashark_garbage` crate contains the garbage collector for the YavaShark engine.

It uses Reference Counting to manage the memory of the engine. Cycles can be detected and collected. Currently, it is not multithreaded, but it is designed to be so.

Why doesn't it collect Javascript itself though?