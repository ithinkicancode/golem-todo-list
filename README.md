# TODO app on Golem Cloud

I assume you have set up Rust's toolchain and installed `cargo-component`. If not, please refer to [Golem Cloud documentation](https://preview1.golem.cloud/learn/rust) for instructions.

Then upload the Wasm binary and run it on Golem Cloud (skip to step 6 if you have already set up Golem CLI):

1. Download the latest version of Golem CLI by [signing up](https://golem.cloud/sign-up) for the Developer Preview.
2. Unzip the bundle to a directory.
3. Define a shell alias to the Golem CLI for convenience. For example:

  ```bash
  alias golem='{path-to-directory}/golem-cli/bin/golem'
  ```

4. Run `golem account get` to go through the authorization process if you haven't done so.
5. `cd` back to our project directory.
6. Run the following command to upload the binary.

  ```bash
  golem component add --component-name todolist target/wasm32-wasi/release/todos.wasm
  ```

7. Then run this command to create an instance of our app.

  ```bash
  golem instance add --instance-name todos-inst-1 --component-name todolist
  ```

8. Define another shell alias to invoke the instance. For example:

  ```bash
  alias todos='golem instance invoke-and-await --instance-name todos-inst-1 --component-name todolist --function $*'
  ```

9. Now let's organzie ✅

  * Run the `count` command to verify that our todo list is currently empty.

  ```bash
  todos golem:todos/api/count --parameters '[]'

  ```

  * META-TODO: finish this readme!

Check out my other Golem projects [here](https://github.com/ithinkicancode/golem-fibonacci) (also a recommended project structure/template) and [here](https://github.com/ithinkicancode/golem-wordle). Have fun!
