import { metadata_collection } from "./mock/metadata_collection.js";
import * as requests from "./requests.js";

const instanceName = process.argv[2];
const noSleep = process.argv[3];

const { instance, dump } = typeof instanceName == "string" && instanceName.length > 0 ? { instance: instanceName, dump: false } : { instance: "cli", dump: true };

const sleep = typeof noSleep == "string" && noSleep.toLowerCase() == "no" ?
    (_) => Promise.resolve() :
    (secs) => new Promise((resolve) => setTimeout(resolve, secs * 1_000));

async function main() {
    // create
    let tokens = await Promise.all(metadata_collection.map(async meta => {
        const token = await requests.create(instance, meta);

        return token;
    }));

    if (dump) requests.dump(instance);

    // update
    tokens = await Promise.all(tokens.map(async oldtoken => {
        await sleep(Math.random() * 10);
        return requests.update(instance, oldtoken, { updatedAt: `${Date.now()}` });
    }));

    if (dump) requests.dump(instance);

    // refresh
    await Promise.all(tokens.filter(info => !info.invalid).map(async (token, index) => {
        if (index == 0 && (instance == "cli" || instance == "A")) {
            await requests.remove(instance, token);
        }
        await sleep(Math.random() * 3);
        return requests.update(instance, token);
    }));

    if (dump) {
        await sleep(1);
        requests.dump(instance);
    }

}

console.log("Start", instance);
main()
    .then(() => console.log("  ***   ", instance, "finished   ***"))
    .catch(e => {
        console.error(instance, "aborted: ", e.message);
        process.exit(1);
    });
