/-------------------------------------\
|    Kendryte OpenOCD MacOS Binary    |
|           version: 0.2.2            |
|          data: 2019/01/17           |
\-------------------------------------/


===============================================================================
                                  Requirement
===============================================================================

Install libusb library:

  ```
  brew install libusb
  ```

===============================================================================
                                  Quickstart
===============================================================================

We have provided a template configuration for J-Link in `tcl/kendryte.cfg`,
if you are using other JTAG emulators, please modify it yourself.

We added a command line argument '-m', it can set the debug mode, now you have
two options, `-m0` for debugging core 0 while `-m1` for debugging core 1, and 
the `-m0` is the default option.

You can start the openocd simply by using the following command:
  
  ./bin/openocd -f ./tcl/kendryte.cfg -m0

After OpenOCD startup, connect GDB with

  (gdb) target remote :3333

then start your debugging! :)

===============================================================================
                                  Bug report
===============================================================================

If you have some problems, you can open a issue at

  https://github.com/kendryte/openocd-kendryte/issues
   
Thank you for your use.
