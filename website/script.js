function recordNewSignal() {
    const name = prompt("Please enter signal name:");
    alert(`Signal '${name}' created`);
}

function replaySignal(button, id) {
    const image = button.children[0];
    image.src = "sending.svg";
    setTimeout(() => {
        image.src = "play.svg";
    }, 100);
}

function renameSignal(id) {
    const name = document.querySelector(`.card:nth-child(${id})>h1`).textContent;
    const new_name = prompt("Please enter new signal name:", name);
    if (new_name ?? "" !== "") {
        document.querySelector(`.card:nth-child(${id})>h1`).textContent = new_name;
    }
}

function downloadSignal(id) {
    const name = document.querySelector(`.card:nth-child(${id})>h1`).textContent;
    downloadFile(`${name}.txt`, "010101010101110");
}

function deleteSignal(id) {
    const name = document.querySelector(`.card:nth-child(${id})>h1`).textContent;
    if (confirm(`Do you really want to delete signal ${name}?`)) {
        alert(`Signal '${name}' deleted`);
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