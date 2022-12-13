import * as requests from "./requests.js";

const instance = "validate";

async function main() {
    let token = await requests.create(instance, { answer: 42 });
    await requests.dump(instance);
    await requests.validate(instance, token);
}

console.log("Start", instance);
main()
    .then(() => console.log("  ***   ", instance, "finished   ***"))
    .catch(e => {
        console.error(instance, "aborted: ", e.message);
        process.exit(1);
    });
