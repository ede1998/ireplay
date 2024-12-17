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
    if (new_name ?? "" != "") {
        document.querySelector(`.card:nth-child(${id})>h1`).textContent = new_name;
    }
}

function deleteSignal(id) {
    const name = document.querySelector(`.card:nth-child(${id})>h1`).textContent;
    alert(`Signal '${name}' deleted`);
}