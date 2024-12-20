class Signal {
    constructor(id, name, curve) {
        this.id = id;
        this.name = name;
        this.curve = curve;
        this.#appendSignal();
    }

    async replay() {
        const image = this.htmlElement.querySelector(".tools>button>img");
        image.src = "sending.svg";
        setTimeout(() => {
            image.src = "play.svg";
        }, 100);

        const request_url = `${window.location.origin}/signals/${this.id}`;
        try {
            await fetch(request_url, { method: "PUT" });
        } catch (error) {
            console.log("Failed to replay signal");
        }
    }

    async rename() {
        const new_name = prompt("Please enter new signal name:", this.name);
        if ((new_name ?? "") === "") {
            return;
        }

        const request_url = `${window.location.origin}/signals/${this.id}`;
        try {
            await fetch(request_url, { method: "POST", body: new_name });
            this.htmlElement.querySelector('h1').textContent = new_name;
        } catch (error) {
            console.log("Failed to rename signal");
        }
    }

    download() {
        downloadFile(`${this.name}.txt`, JSON.stringify(this.curve));
    }

    #appendSignal() {
        const id = this.id;
        const signal_card = `<article class="card" id="signal-${id}">
            <h1>${this.name}</h1>
            <img src="${this.#createSvg()}" alt="IR signal curve">
            <div class="tools">
                <button onclick="ALL_SIGNALS.replay(${id});"><img class="replay" src="play.svg" alt="Replay IR signal" /></button>
                <button onclick="ALL_SIGNALS.rename(${id});"><img src="edit.svg" alt="Rename IR signal" /></button>
                <button onclick="ALL_SIGNALS.download(${id});"><img src="save.svg" alt="Download IR signal" /></button>
                <button onclick="ALL_SIGNALS.delete(${id});"><img src="trash.svg" alt="Delete IR signal" /></button>
            </div>
        </article>`;

        const gallery = document.querySelector(".cards");
        const add_button = gallery.querySelector(".card:last-child")
        add_button.insertAdjacentHTML("beforebegin", signal_card)
    }

    #createSvg() {
        const image_width = 1000;
        const image_height = 500;
        const high = 5;
        const low = Math.round(image_height - high);
        const x = (i) => Math.round(image_width * i / (this.curve.length + 1));
        const y = (v) => (v == 1) ? high : low;

        const points = this.curve
            .entries()
            .flatMap(([i, v]) => [`${x(i)},${y(v)}`, `${x(i + 1)},${y(v)}`])
            .toArray();

        const code = `<svg width="${image_width}" height="${image_height}" xmlns="http://www.w3.org/2000/svg">
            <polyline points="${points.join(" ")}" style="fill:none;stroke:red;stroke-width:4" />
        </svg>`;

        const blob = new Blob([code], { type: "image/svg+xml" });
        const link = window.URL.createObjectURL(blob);

        return link;
    }

    get htmlElement() {
        const id = `signal-${this.id}`;
        const element = document.getElementById(id);
        if (element === null || element === undefined) {
            console.warn(`Could not find HTML element for signal ${this.id} (${this.name})`);
        }
        return element;
    }
}


class AllSignals {
    async initialize() {
        const request_url = `${window.location.origin}/signals`;
        let signal_data;

        try {
            const response = await fetch(request_url);
            signal_data = await response.json();
        } catch (error) {
            console.log("Failed to retrieve signals");
            return {};
        }

        for (const [id, signal] of Object.entries(signal_data)) {
            this[id] = new Signal(id, signal.name, signal.curve);
        }
    }

    async replay(id) {
        await this[id].replay();
    }

    async rename(id) {
        await this[id].rename();
    }

    download(id) {
        this[id].download();
    }

    async record() {
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
            this[id] = new Signal(id, signal.name, signal.curve);
        } catch (error) {
            console.log("Failed to create new signal");
            return;
        }
    }

    async delete(id) {
        const signal = this[id];
        if (!confirm(`Do you really want to delete signal ${signal.name}?`)) {
            return;
        }

        const request_url = `${window.location.origin}/signals/${id}`;
        try {
            await fetch(request_url, { method: "DELETE" });
            alert(`Signal '${signal.name}' deleted`);
        } catch (error) {
            console.log("Failed to delete signal");
        }

        signal.htmlElement.remove();
        delete this[id];
    }
}


function downloadFile(name, contents, mime_type) {
    mime_type = mime_type || "text/plain";

    const blob = new Blob([contents], { type: mime_type });

    const download_link = document.createElement('a');
    download_link.download = name;
    download_link.href = window.URL.createObjectURL(blob);
    download_link.onclick = function (_) {
        // revokeObjectURL needs a delay to work properly
        const that = this;
        setTimeout(() => {
            window.URL.revokeObjectURL(that.href);
        }, 1500);
    };

    download_link.click();
    download_link.remove();
}

const ALL_SIGNALS = new AllSignals();