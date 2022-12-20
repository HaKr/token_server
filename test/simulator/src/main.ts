import { CommandLine } from "./clap.ts";
import { Simulator } from "./simulator.ts";

const options = {
  help: false,
  name: "sim",
  include_errors: false,
  random_wait: 0,
  shutdown_after: 0,
};

CommandLine.parse(options).async_map_or_else(
  (err) => {
    console.error("\n");
    Simulator.LOGGER.error(err.toString());
    showHelp(console.error);
    Deno.exit(1);
  },
  async (options) => {
    Simulator.LOGGER.debug(options);

    if (options.help) {
      showHelp(console.log);
      Deno.exit(0);
    }

    const simulator = new Simulator(
      options.name,
      options.include_errors,
      options.random_wait,
    );

    if (options.shutdown_after > 0) {
      setTimeout(
        () => simulator.shutdownServer(),
        options.shutdown_after * 1000,
      );
    }

    await simulator.run()
      .then(
        (res) => {
          res.map_or_else(
            (err) => Simulator.LOGGER.error(err),
            () => Simulator.LOGGER.info("Simulation finished"),
          );
        },
        (err) => Simulator.LOGGER.error(err),
      );
  },
);

function showHelp(logger: (...args: unknown[]) => void) {
  logger(
    `
    
  Usage: LOG=tokenclient=[LEVEL],simulator=[LEVEL],scheduler=[LEVEL] deno run --allow-net --allow-env src/main.ts [OPTIONS]

  LEVEL = trace | debug | info | warn | error | off
    Verbosity of the specified module

  Options:
    --help
        Show this help

    --name STRING
        Name thie simulation, default: sim

    --include-error
        Add this argument to generate some errors

    --random-wait INTEGER
        Pass a value to specify the maximum number of seconds to wait between simulation tasks.
        When no positive integer is specified, all tasks are performed without any added delay.
`,
  );
}
