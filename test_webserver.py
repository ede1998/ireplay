from flask import Flask, redirect

app = Flask(__name__, static_folder="website")

@app.route('/')
def hello():
    return redirect("website/index.html", code=302)