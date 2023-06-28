```
oco [edit]
  --vocabulary=gnu.ls
  -I|--input-dialect=gnu  # default
  -O|-output-dialect=gnu  # default
  -i|--input=FILE         # default: stdin
  -o|--output=FILE        # default: stdout
  -s|--script             # Use long format
  -f|--file               # Use file
  <(cat EOF
  set all # ensure --all flag is set (don't add if already there)
  set ignore=foo # ensure --ignore flag is set to 'foo'
              # if --ignore flag already exists, value would change
              # if multiple --ignore flags exist, the first one would be changed
  set v   # flag will be -v in GNU dialect, since it is single letter
  add v # adds flag -v even if already present
  add ignore=foo # adds flag --ignore with value 'foo' even if already present
  add ignore='long value' # quoted value
  remove ignore # removes all --ignore flags
  remove v # removes flag all -v flags
```

For usage directly in arguments, add a semicolon (`;`) character as statement
separator:
```
oco -- remove v; set all; add ignore='filename with spaces.txt'
```

You can add repeated values using `radd`:

```
radd f=file1 file2 'filename with spaces.txt'
radd ignore=*.png *.jpg
```

Would be added as:
```
-f file1 -f file2 -f 'filename with spaces.txt'
--ignore='*.png' --ignore='*.jpg'
```

Dialects can deal with multiple (unquoted) values in different ways. for
instance, the gnu and posix dialects, can use this as a hack to get multiple
positional values after a flag, e.g.:
```
  set c=apple banana cherry
  set foo=bar baz1 baz2 baz3
```
Would be translated by the GNU dialect to:
```
  -c apple banana cherry
  --foo=bar baz1 baz2 baz3
```

There is also a simplified (shorter) grammar that you can use:
```
oco
  --
  a          # set a
  all        # set all
  sort=size  # set sort=size
  +v         # add v
  +verbose   # add verbose
  +sort=size # add sort=size
  -v         # remove ver
  -verbose   # remove verbose
  
  # Repeated Add is supported
  ++ignore="*.png *.jpg 'filename with spaces.txt'"
  
  # Long values are supported:
  +sort='long value' # add sort=long value
  
  # Multiple values are supported:
  +sort/="'quoted long value' 'another value' last"
```

