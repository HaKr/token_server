const SERVER = "http://127.0.0.1:3666";
const ENDPOINT_TOKEN = `${SERVER}/token`;
const ENDPOINT_DUMP = `${SERVER}/dump`;
const ENDPOINT_NONEXISTING = `${SERVER}/doesnotexist`;

const headers = {
    "Content-Type": "application/json"
};

export async function create(instance, meta) {
    if (meta != null) meta.instance = instance;
    const response = await fetch(`${ENDPOINT_TOKEN}?instance=${instance}`, {
        method: "POST",
        headers,
        body: JSON.stringify({ meta })
    });

    if (response.ok) {
        let token = await response.text();
        if (token.startsWith("ERROR")) {
            throw new Error(token.substring(7));
        } else {
            return { created: Date.now(), token: await response.text() };
        }
    } else {
        throw new Error(await response.text())
    }
}

export async function update(instance, tokenInfo, meta) {
    const { token, created, format, log } = analyse(tokenInfo, instance, meta ? `UPDATE ${metaInfo(meta)}` : "REFRESH");

    const body = meta ? { token, meta } : { token };
    const response = await fetch(`${ENDPOINT_TOKEN}?d=${instance}`, {
        method: "PUT",
        headers,
        body: JSON.stringify(body)
    });

    if (response.ok && response.headers.get('content-type') == 'application/json') {
        let info = await response.json();
        if (info.Ok) {
            log("success", metaInfo(info.Ok.meta));
            return { created: Date.now(), token: info.Ok.token };
        } else {
            log("failed", info.Err);
            return { created, invalid: true };

        }
    } else {
        throw new Error(format(`canceled: ${await response.text()}`));
    }
}

export async function remove(instance, tokenInfo) {
    const { token, format, log } = analyse(tokenInfo, instance, "DELETE");

    let response = await fetch(`${ENDPOINT_TOKEN}?instance=${instance}`, {
        method: "DELETE",
        headers,
        body: JSON.stringify({ token })
    });

    if (response.ok) {
        log("success");
    } else {
        throw new Error(format(`failed: ${await response.text()}`));
    }
}

export function dump(instance) {

    return fetch(`${ENDPOINT_DUMP}?d=${instance}`, {
        method: "HEAD"
    });
}

export async function nonexisting(instance) {
    const response = await fetch(`${ENDPOINT_NONEXISTING}?d=${instance}`, {
        method: "GET"
    });
    if (!response.ok) {
        throw new Error(`${response.status} ${response.statusText}`);
    }

}

function analyse(tokenInfo, instance, label) {
    const { created, token } = tokenInfo;
    const lifetime = Math.round((Date.now() - created) / 1000);
    const format = (result, ...args) => formatLabel(instance, lifetime, `${label} ${result}${args.length > 0 ? ": " : ""}`);
    const log = (result, ...args) => console.log(format(result, ...args), ...args);

    return { token, created, format, log };

}

function metaInfo(meta) {
    return `${meta.lastName ? meta.lastName : meta.year ? meta.year : ""}` +
        `${(meta.lastName || meta.year) && meta.updatedAt ? ", " : ""}` +
        `${meta.updatedAt ? `updatedAt: ${meta.updatedAt}` : ''}`;
}

function formatLifetime(lifetime) {
    return lifetime < 10 ? lifetime < 1 ? " 0" : `0${lifetime}` : `${lifetime}`
}

function formatLabel(instance, lifetime, msg) {
    return `[${instance}, ${formatLifetime(lifetime)}s] ${msg}`;
}