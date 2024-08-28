import argparse
import io

import numpy as np
import torch
import yaml
from PIL import Image
from flask import Flask, request, jsonify
from werkzeug import exceptions

from model import DirectMHPInfer
from utils.general import scale_coords
from utils.augmentations import letterbox

device = torch.device("cuda:0" if torch.cuda.is_available() else "cpu")
model = DirectMHPInfer(weights="weights/agora_m_best.pt", device=device)

app = Flask(__name__)
config = {
    "img_size": 640,
    "stride": model.model.stride.max().item(),
    "prediction": ["x1", "y1", "x2", "y2", "conf", "class", "pitch", "yaw", "roll"],
}


@app.route("/predict", methods=["POST"])
def predict():
    img, old_shape = preprocess_img(
        extract_img(request).read(), config["img_size"], config["stride"]
    )

    img = torch.from_numpy(img).to(device=device)
    img = img / 255.0

    img = img[None]

    try:
        out = model(img)[0]
    except Exception as e:
        raise exceptions.InternalServerError(f"Error in inference: {e}")

    out[:, :4] = scale_coords(img.shape[2:], out[:, :4].clone().detach(), old_shape[:2])

    out = [t.cpu().detach().numpy().tolist() for t in out]
    out = [dict(zip(config["prediction"], pred)) for pred in out]

    return jsonify({"prediction": out})


def extract_img(request):
    if "image" not in request.files:
        raise exceptions.BadRequest("Missing image parameter.")

    img = request.files["image"]
    if img.filename == "":
        raise exceptions.BadRequest("Invalid image given.")

    return img


def preprocess_img(img, new_img_size, stride, auto=True):
    try:
        img = np.array(Image.open(io.BytesIO(img)).convert(mode="RGB"))
    except Exception as e:
        raise exceptions.InternalServerError(f"Couldn't convert image to ndarray: {e}")

    old_shape = img.shape

    # padded resize
    img = letterbox(im=img, new_shape=new_img_size, stride=stride, auto=auto)[0]

    # convert
    img = img.transpose((2, 0, 1))[::-1]  # HWC to CHW
    img = np.ascontiguousarray(img)

    return img, old_shape


if __name__ == "__main__":
    print("Starting DirectMHP service...")

    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default="8080", help="Port to listen on")
    parser.add_argument("--host", type=str, default="0.0.0.0", help="Host to listen on")
    parser.add_argument(
        "-i", "--img-size", type=int, default=640, help="Image resize size"
    )
    parser.add_argument(
        "-s",
        "--stride",
        type=int,
        default=model.model.stride.max().item(),
        help="Image resize stride",
    )

    args = parser.parse_args()
    config["img_size"] = args.img_size
    config["stride"] = args.stride

    app.run(debug=False, host=args.host, port=args.port)
