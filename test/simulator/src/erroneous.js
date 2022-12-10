import * as requests from "./requests.js";

const instance = "err";

// { token: "made-up", deepThought: { answer: 42 } }

async function main() {
    // create without meta
    await requests.create(instance, null).then(token => console.log("Token=", token), console.error);

    // create with invalid payload type
    await requests.create_invalid(instance, null).then(token => console.log("Token=", token), console.error);

    // invalid endpoint
    await requests.nonexisting(instance).then(() => console.log("succeeded"), console.error);

}

console.log("Start", instance);
main()
    .then(() => console.log("  ***   ", instance, "finished   ***"))
    .catch(e => {
        console.error(instance, "aborted: ", e.message);
        process.exit(1);
    });
