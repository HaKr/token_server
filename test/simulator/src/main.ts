import { CommandLine, None, Option, Simulator, Some } from "./deps.ts";

const options = {
  name: "sim",
  include_errors: false,
  random_wait: 0,
  shutdown_after: 0,
};

CommandLine.parse(options, showHelp)
  .map(
    async (options) => {
      Simulator.LOGGER.debug(options);

      const simulator = Simulator.create({
        name: options.name,
        includeErrors: options.include_errors,
        randomWait: options.random_wait,
      });

      const handle: Option<number> = (options.shutdown_after < 1)
        ? None()
        : Some(setTimeout(
          () => simulator.shutdownServer(),
          options.shutdown_after * 1000,
        ));

      await simulator
        .run()
        .mapOrElse(
          (err) => Simulator.LOGGER.error(err),
          () => Simulator.LOGGER.info(`Simulation ${options.name} finished`),
        );

      handle.map(clearTimeout);
    },
  );

function showHelp(opts: typeof options) {
  console.log(`  
Usage: LOG=[default LEVEL],tokenclient=[LEVEL],simulator=[LEVEL],scheduler=[LEVEL] deno run --allow-net --allow-env src/main.ts [OPTIONS]

LEVEL = trace | debug | info | warn | error | off
  Verbosity of the specified module

Options:
  --help
      Show this help

  --name STRING
      Optional name for the simulation, default: sim (${opts.name})

  --include-errors 
      Add this argument to generate some errors (${opts.include_errors})

  --random-wait INTEGER
      Pass a value to specify the maximum number of seconds to wait between simulation tasks.
      When no positive integer is specified, all tasks are performed without any added delay.
      (${opts.random_wait > 0 ? `< ${opts.random_wait}s` : "no delay"})

  --shutdown-after INTEGER
      Pass a value to specify the number of seconds after which to shutdown the token server.
      When no positive integer is specified, the token server will remain active.
      (${opts.shutdown_after > 0 ? `${opts.shutdown_after}s` : "no shutdown"})
`);
}
