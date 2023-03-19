ipdmsw
======

This is an example program and utility library for the iPDM56 module.

I will publish code improvements and ideas once I figure out good ways to
generalize things.

I will try to keep any vehicle specific code to myself as it always becomes a
hairy mess with insane design choices.

- celeron55


Usage
-----

Copy the the ipdmsw folder into your project and rename it along with the .ino
file.

Configure your iPDM56 version in `ipdm_version.h` to get correct pin mappings.

Edit the .ino file, `param_def.h` and others to make it do what you wish it to
do.


Common mistakes
---------------

Remember to always use `ipdm::digitalRead`, `ipdm::digitalWrite` and `ipdm::pinMode` instead of `digitalRead`, `digitalWrite` and `pinMode`, as the latter don't support the I/O extender pins.
* Not doing this is a super common mistake for myself too! -celeron55

