import { metadata_collection } from "./mock/metadata_collection.js";
import * as requests from "./requests.js";

const instanceName = process.argv[2];
const randomSleep = process.argv[3];

const DEFAULT_INSTANCE = "cli";

const { instance, dump } = typeof instanceName == "string" && instanceName.length > 0 && instanceName != DEFAULT_INSTANCE ? { instance: instanceName, dump: false } : { instance: DEFAULT_INSTANCE, dump: true };

const { sleep, doSleep } = typeof randomSleep == "string" && randomSleep.toLowerCase() == "yes" ?
    {
        sleep: (secs) => new Promise((resolve) => setTimeout(resolve, Math.random() * secs * 1_000)), doSleep: true
    } :
    { sleep: (_) => Promise.resolve(), doSleep: false };

console.debug(`Instance: ${instance}; dump: ${dump}; sleep? ${doSleep}`)

async function main() {
    // create
    let tokens = await Promise.all(metadata_collection.map(async meta => {
        const token = await requests.create(instance, meta);

        return token;
    }));

    if (dump) requests.dump(instance);

    // update
    tokens = await Promise.all(tokens.map(async oldtoken => {
        await sleep(10);
        return requests.update(instance, oldtoken, { updatedAt: `${Date.now()}` });
    }));

    if (dump) requests.dump(instance);

    // refresh
    await Promise.all(tokens.filter(info => !info.invalid).map(async (token, index) => {
        if (index == 0 && (instance == DEFAULT_INSTANCE || instance == "A")) {
            await requests.remove(instance, token);
        }
        await sleep(3);
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
