```
oco
  --vocabulary=gnu.ls
  -I|--input-dialect=gnu  # default
  -O|-output-dialect=gnu  # default
  -i|--input=FILE         # default: stdin
  -o|--output=FILE        # default: stdout
  -l|--script             # Use long-form script
  -f|--file=FILE          # Use file for script (otherwise arguments are used)
  <(cat EOF
  set ignore=foo # ensure --ignore flag is set to 'foo'
              # if --ignore flag already exists, value would change
              # if multiple --ignore flags exist, the last one would be changed
  set all # ensure --all flag is set and remove all arguments (for first instance if multiple are found)
  set v   # flag will be -v in GNU dialect, since it is single letter
  add v # adds flag -v even if already present
  add ignore=foo # adds flag --ignore with value 'foo' even if already present
  add ignore='long value' # quoted value
  remove ignore # removes all --ignore flags
  remove v # removes flag all -v flags
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

You can append a suffix to a string:

```
append name=-suffix
append name='-more names'
```

Would be translated to:
```
--name='current-suffix-more names'
```

You can also append with an delimiter (when existing value is non-empty):
```
append ','  empty-list=value1 value2
append ','  nonempty-list=value1 value2
append '; ' with-spaces=value1 value2
```

Would be translated to:
```
--empty-list=value1,value2
--nonempty-list='current,value1,value2'
--with-spaces='current; value1; value2'
```

Dialects can deal with multiple (unquoted) values in different ways. for
instance, the gnu and posix dialects, can use this as a hack to get multiple
positional values after a flag, e.g.:
```
  set c=apple banana cherry
  add foo=bar baz1 baz2 baz3
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
  -v         # remove all v
  -verbose   # remove all verbose
  
  # Add suffix to name
  name+='-suffix' 
  
  # Add value1,value2 and prepend delimeter ',' unless name is empty
  name+','=value1,value2
  
  # You can add any delimeter inside the quotes, even with spaces:
  # For isntance, the following would change name from "a + b" to "a + b + c".
  name+' + '=c
  
  # The following common delimeter characters are accepted without quotes:
  # , ; : & |
  name+,=value1,value2
  path+:=/usr/local/bin:/opt/addon/bin
  
  # Repeated Add is supported
  ++ignore="*.png *.jpg 'filename with spaces.txt'"
  
  # Long values are supported:
  +sort='long value' # add sort='long value'
  
  # Multiple values are supported:
  sort/="'quoted long value' 'another value' last"
  +sort/="'quoted long value' 'another value' last"
```

