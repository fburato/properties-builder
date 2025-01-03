# properties-builder

`properties-builder` is a command line utility which allows to manipulate `.properties` file with environment variables. It takes inspiration from the [`start-kafka.sh`](https://github.com/wurstmeister/kafka-docker/blob/master/start-kafka.sh) of the [wurstmeister/kafka](https://hub.docker.com/r/wurstmeister/kafka/) docker image as a way to make kafka more container friendly, but its principles have been generified for broader applications. It builds as a static binary to ensure that no additional dependencies are required to run the application.

## Installation

From a system with the rust toolchain installed, run `cargo install https://github.com/fburato/properties-builder`.

## Command line reference

```
Generate a properties file from existing properties overriding the values from environment variables and removing all overrides

Usage: properties-builder [OPTIONS] --prefix <PREFIX> [FILE]

Arguments:
  [FILE]
          Original property file to read for override. If not provided, stdin is read instead

Options:
      --output-file <OUTPUT_FILE>
          If provided, output the properties file to a file instead of standard output

  -p, --prefix <PREFIX>
          Specifies the prefix for environment variables to use for overrides and generation

  -s, --spring
          When selected, uses spring style properties replacement, i.e. converts '.,-' into '_' and capitalises all text. Incompatible with -r or --replacement options.

          For example, passing '--prefix PREFIX_ --spring' causes environment variable PREFIX_FOO=bar' to be interpreted as 'foo=bar'

  -r, --replacement <REPLACEMENT>
          Specifies character replacement for case-insensitive interpretation in the format 'c#str'.

          For instance, passing '--prefix PREFIX -r .#_ -r _#__' causes environment variable PREFIX_foo_bar__baz=foobar to be interpreted as 'foo.bar_baz=foobar'. To replace character '-' use '\-' (e.g. '\-#__')

      --empty-input
          If passed, no input file nor stdin is read for override and only properties generated from the environment are added to the output

  -h, --help
          Print help (see a summary with '-h')
```

## Example usage

- Update in place the `application.properties` file, using the Spring substitution style and using the `PROP_` prefix to select environment variables: `properties-builder --spring --prefix PROP_ --output-file application.properties application.properties`

  ```properties
  # application.properties
  foo=test
  bar.baz=foobar
  ```

  environment variables:
  
  ```
  PROP_FOO=new test
  PROP_BAZ_BAZ=new value
  ```
  
  output:

  ```properties
  # application.properties
  foo=new test
  bar.baz=foobar
  baz.baz=new value
  ```

- Output to standard output the result of processing of `test.properties` using the [HOCON](https://github.com/lightbend/config) style replacements `properties-builder --prefix CONFIG_FORCE_ -r '.#_' -r '\-#__' -r '_#___' test.properties`

  ```properties
  # test.properties
  a.b-c_d.e=foo
  b_d=bar
  ```
  
  environment variables:

  ```
  CONFIG_FORCE_a_b__c___d_e=bar
  CONFIG_FORCE_d__b_c=baz
  ```
  
  output:

  ```properties
  # test.properties
  a.b-c_d.e=bar
  b_d=bar
  d-b.c=baz
  ```

## Notes

Using explicit replacement, it's important to note that the replacement of separator in environment variables follows a greedy strategy, replacing the longest replacement string before the shorter ones. For example, using the HOCON replacement `-r '.#_' -r '\-#__' -r '_#___'`, the environment variable `CONFIG_FORCE_a_____b` is interpreted as the key `a_-b` rather than `a.....b` or `a--.b`, so ensure that an appropriate separator and replacement strategy is used in case separator for keys are ambiguous.
  