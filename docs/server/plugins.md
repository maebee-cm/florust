# Server-side plugins

Florust utilizes server-side plugins to parse data that is submitted to the server. For now, Florust's server implementation only allows for the graphing of `u64`, `i64`, and `f64`, and as such, any plugin responsible for manager data sources, must ultimately spit out one of those 3 types.

For the sake of convenience, Florust, by default, offers some plugins that allow it to process plain numerical data that doesn't need to processing. That is, data that is big endian encoded bytes that represent `u64`, `i64`, or `f64` data. Those plugins have the ids: `FlorustDefaultIIntegerDataManager`, `FlorustDefaultUIntegerDataManager` and `FlorustDefaultFloatDataManager`, corresponding respectively to the data types mentioned earlier.

The existence of these default plugins should be appropriate for most usages that are logging numerical data, however, if your data requires some processing before it can be turned into one of the 3 data types that Florust supports, a custom plugin will be necessary. The default plugins are an excellent starter example for what a bare bones minimal plugin would look like. They can all be found in the [default_plugins.rs](/florust_server/src/default_plugins.rs) file, under the `src` folder inside of `florust_server`.

## Custom plugins

Creating custom plugins is very simple, and the steps for which are as follows:

1. Determine what data type the plugin will create with the data that it is given (`i64`, `u64`, or `f64`).
2. Create a struct that implements `IIntegerDataSourceManager`, `UIntegerDataSourceManager`, or `FloatDataSourceManager` respectively depending on what data type it will be creating.
3. Create a function of type `CreateIIntegerDataSourceManager`, `CreateUIntegerDataSourceManager`, or `CreateFloatDataSourceManager`, that matches what trait the struct implements. While you can name your function anything, its suggested that you name the function `create_iinteger_data_source_manager`, `create_uinteger_data_source_manager`, or `create_float_data_source_manager` respective to what data source manager your struct implements,
4. Compile the plugin as a dynamic library.
5. In the same working directory that the Florust server would be running in, create a folder called `plugins`
6. Create a folder inside `plugins`, ideally the folder name should reflect the name of your plugin.
7. Create `plugin.toml` file inside your folder, this will be the file that holds info for how your plugin should be configured. Formatting for this config file is described later in this document.
8. Put your dynamic library in the same folder as the `plugin.toml` file.

## Config file

The config file is a TOML file, it requires one section, the `plugin` section. You can however, should your plugin need it, require extra parameters be included in your config file by the user. Should this be the case, Florust can pass those parameters to your plugin during the plugin creation.

### Required parameters

All required parameters must be placed in a section labeled `plugin`. The required parameters are described below.

| name        | description                                                  | default value        | accepted values |
| ----------- | ------------------------------------------------------------ | -------------------- | --------------- |
| name        | name of the plugin                                           | N/A                  | any             |
| lib         | name of the file                                             | N/A                  | any             |
| max_data    | maximum number of data points stored per data source         | 10                   | any             |
| data_type   | the type of data this plugin will be reporting               | N/A                  | [i64, u64, f64] |
| create_func | name of the function that will be used to create the manager | depends on data_type | any             |
| need_config | whether or not the plugin needs the config file              | false                | [true, false]   |

### Example config file

An example of a config file, with explicit values for all values which have a default is given below. This example also includes an extra configuration section to demonstrate an example of a plugin that requires more information to be supplied by the user.

```toml
[plugin]
name = "sample_plugin"
lib = "sample_plugin.so"
max_data = 10
data_type = "i64"
create_func = "create_iinteger_data_source_manager"

[exampleExtraSection]
foo = "bar"
foobar = "baz"
baz = 3
```
