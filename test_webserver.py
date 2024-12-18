from flask import Flask, redirect, request
import random

app = Flask(__name__, static_folder="website")

def invert(value: int) -> int:
    if value == 0:
        return 1
    return 0

def generate_signal(name: str) -> dict[str, any]:
    length = 1000
    flip_count = random.randint(10, 100)
    flip_positions = {random.randrange(length) for _ in range(flip_count)}
    value = random.choice([0, 1])

    return {
        "name": name,
        "curve": [(value := invert(value)) if i in flip_positions else value for i in range(length)],
    }

ALL_SIGNALS: dict[int, dict[str, any]] = {
    id: generate_signal(name) for id, name in enumerate(["Frank", "Peter", "Harald", "Matthias", "Mark", "Lars"])
}
NEXT_ID = len(ALL_SIGNALS)

def print_signal_ids():
    app.logger.info(f"Available signal ids: {','.join([str(id) for id in ALL_SIGNALS])}")

@app.get("/")
def website():
    return redirect("website/index.html", code=302)

@app.get("/signals")
def signals():
    return ALL_SIGNALS

@app.delete("/signals/<int:id>")
def delete_signal(id: int):
    del ALL_SIGNALS[id]
    return "", 200

@app.post("/signals/<int:id>")
def rename_signal(id: int):
    new_name = request.get_data(as_text=True)
    ALL_SIGNALS[id]["name"] = new_name
    return "", 200

@app.post("/signals")
def record_new_signal():
    global NEXT_ID
    name = request.get_data(as_text=True)
    signal = generate_signal(name)
    ALL_SIGNALS[NEXT_ID] = signal
    NEXT_ID += 1
    print_signal_ids()
    return { NEXT_ID - 1: signal}, 201

@app.put("/signals/<int:id>")
def replay_signal(id: int):
    print(f"Replaying signal {ALL_SIGNALS[id]["name"]}")
    return "", 200