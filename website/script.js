let ALL_SIGNALS = {}


async function loadAllSignals() {
    const request_url = `${window.location.origin}/signals`;
    try {
        const response = await fetch(request_url);
        const signal_data = await response.json();
        return signal_data;
    } catch (error) {
        console.log("Failed to retrieve signals");
        return {};
    }
}

function appendSignal(id, signal) {
    const signal_card = `<article class="card" id="signal-${id}">
        <h1>${signal.name}</h1>
        <img src="recording.svg" alt="IR signal curve">
        <div class="tools">
            <button onclick="replaySignal(this, ${id});"><img src="play.svg" alt="Replay IR signal" /></button>
            <button onclick="renameSignal(${id});"><img src="edit.svg" alt="Rename IR signal" /></button>
            <button onclick="downloadSignal(${id});"><img src="save.svg" alt="Download IR signal" /></button>
            <button onclick="deleteSignal(${id});"><img src="trash.svg" alt="Delete IR signal" /></button>
        </div>
    </article>`;

    const gallery = document.querySelector(".cards");
    const add_button = gallery.querySelector(".card:last-child")
    add_button.insertAdjacentHTML("beforebegin", signal_card)
}

async function initialize() {
    ALL_SIGNALS = await loadAllSignals();
    for (const [id, signal] of Object.entries(ALL_SIGNALS)) {
        appendSignal(id, signal);
    }
}

async function recordNewSignal() {
    const name = prompt("Please enter signal name:");
    const request_url = `${window.location.origin}/signals`;
    try {
        const response = await fetch(request_url, {
            method: "POST",
            headers: { 'Content-Type': 'text/plain' },
            body: name,
        });
        const signal_with_id = await response.json();

        if (Object.entries(signal_with_id).length !== 1) {
            console.log(`Unexpected format for ${JSON.stringify(signal_with_id)}`);
            return;
        }
        const [id, signal] = Object.entries(signal_with_id)[0];

        alert(`Created signal '${name}'`);
        ALL_SIGNALS[id] = signal;
        appendSignal(id, signal);
    } catch (error) {
        console.log("Failed to create new signal");
        return;
    }
}

async function replaySignal(button, id) {
    const image = button.children[0];
    image.src = "sending.svg";
    setTimeout(() => {
        image.src = "play.svg";
    }, 100);

    const request_url = `${window.location.origin}/signals/${id}`;
    try {
        await fetch(request_url, { method: "PUT" });
    } catch (error) {
        console.log("Failed to replay signal");
    }
}

async function renameSignal(id) {
    const name = ALL_SIGNALS[id].name;
    const new_name = prompt("Please enter new signal name:", name);
    if ((new_name ?? "") === "") {
        return;
    }

    const request_url = `${window.location.origin}/signals/${id}`;
    try {
        await fetch(request_url, { method: "POST", body: new_name });
        document.querySelector(`#signal-${id}>h1`).textContent = new_name;
    } catch (error) {
        console.log("Failed to rename signal");
    }
}

function downloadSignal(id) {
    const name = document.querySelector(`.card:nth-child(${id})>h1`).textContent;
    downloadFile(`${name}.txt`, "010101010101110");
}

async function deleteSignal(id) {
    const name = ALL_SIGNALS[id].name;
    if (!confirm(`Do you really want to delete signal ${name}?`)) {
        return;
    }

    const request_url = `${window.location.origin}/signals/${id}`;
    try {
        await fetch(request_url, { method: "DELETE" });
        alert(`Signal '${name}' deleted`);
        document.getElementById(`signal-${id}`).remove();
        delete ALL_SIGNALS[id];
    } catch (error) {
        console.log("Failed to delete signal");
    }
}

function downloadFile(name, contents, mime_type) {
    mime_type = mime_type || "text/plain";

    const blob = new Blob([contents], { type: mime_type });

    const download_link = document.createElement('a');
    download_link.download = name;
    download_link.href = window.URL.createObjectURL(blob);
    download_link.onclick = function (e) {
        // revokeObjectURL needs a delay to work properly
        const that = this;
        setTimeout(() => {
            window.URL.revokeObjectURL(that.href);
        }, 1500);
    };

    download_link.click();
    download_link.remove();
}