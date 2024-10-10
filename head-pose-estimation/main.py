import socket
import struct
import json

import torch
import numpy as np

from model import DirectMHPInfer
from utils.general import scale_coords
from utils.augmentations import letterbox


device = torch.device("cuda:0" if torch.cuda.is_available() else "cpu")
model = DirectMHPInfer(weights="weights/agora_m_best.pt", device=device)
# opt_model = torch.compile(model, mode="default")
opt_model = model

config = {
    "process_id": "directmhp",
    "server_address": "/tmp/gesurease.sock",
    "img_size": 320,
    "stride": model.model.stride.max().item(),
    "prediction":
        ["x1", "y1", "x2", "y2", "conf", "class", "pitch", "yaw", "roll"],
    "debug": False,
}


def preprocess(img: np.ndarray, new_img_size, stride, auto=True):
    old_shape = img.shape

    # padded resize
    img = letterbox(im=img, new_shape=new_img_size,
                    stride=stride, auto=auto)[0]

    # convert
    img = img.transpose((2, 0, 1))[::-1]  # HWC to CHW
    img = np.ascontiguousarray(img)

    return img, old_shape


def to_radiants(pitch_yaw_roll: np.ndarray):
    from math import pi

    if pitch_yaw_roll.shape[0] < 1:
        return pitch_yaw_roll

    # shifter = np.array([-0.5, -0.5, -0.5]).reshape(1, 3)
    # pier = np.array([pi, 2 * pi, pi]).reshape(1, 3)

    # return (pitch_yaw_roll - shifter) * pier
    for i in range(pitch_yaw_roll.shape[0]):
        pitch_yaw_roll[i, 0] = (pitch_yaw_roll[i, 0] - 0.5) * pi
        pitch_yaw_roll[i, 1] = (pitch_yaw_roll[i, 1] - 0.5) * 2 * pi
        pitch_yaw_roll[i, 2] = (pitch_yaw_roll[i, 2] - 0.5) * pi

    return pitch_yaw_roll


def pred(img, w, h):
    # img = np.array(Image.open(io.BytesIO(img)).convert(mode="RGB"))
    img = np.frombuffer(img, np.uint8).reshape(h, w, 3)

    img, old_shape = preprocess(img, config["img_size"], config["stride"])

    img = torch.from_numpy(img).to(device=device)
    img = img / 255.0

    img = img[None]

    if config["debug"]:
        start = time.time()
        out = opt_model(img)[0]
        end = (time.time() - start) * 1000

        print(f"\t\tinference: {end:.1f} ms")
    else:
        out = opt_model(img)[0]

    out[:, :4] = scale_coords(
        img.shape[2:], out[:, :4].clone().detach(), old_shape[:2])
    out[:, 6:] = to_radiants(out[:, 6:])

    out = [t.cpu().detach().numpy().tolist() for t in out]
    out = [dict(zip(config["prediction"], pred)) for pred in out]

    return json.dumps({"prediction": out})


def run():
    img_width_bytes = sock.recv(4)
    img_height_bytes = sock.recv(4)
    data_len_bytes = sock.recv(4)
    if len(data_len_bytes) == 0:
        print("Connection closed, exiting...")
        raise

    img_width = struct.unpack("!I", img_width_bytes)[0]
    img_height = struct.unpack("!I", img_height_bytes)[0]
    data_len = struct.unpack("!I", data_len_bytes)[0]

    if config["debug"]:
        start = time.time()
    img = sock.recv(data_len)
    while len(img) < data_len:
        img += sock.recv(data_len - len(img))

    # print(img)

    if config["debug"]:
        start2 = time.time()

    preds = pred(img, img_width, img_height)

    if config["debug"]:
        end2 = (time.time() - start2) * 1000

    sock.sendall(struct.pack("!I", len(preds)))
    sock.sendall(preds.encode())

    if config["debug"]:
        end = (time.time() - start) * 1000
        print(f"\tipc time: {end - end2:.1f} ms")
        print(f"duration: {end:.1f} ms")


if __name__ == "__main__":
    import time
    import os
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("--socket")

    args = parser.parse_args()

    # Socket file path
    socket_path = args.socket if args.socket else "/tmp/hpe.sock"

    # Remove the socket file if it already exists
    try:
        os.unlink(socket_path)
    except OSError:
        if os.path.exists(socket_path):
            raise

    # Create a Unix domain socket
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)

    # Bind the socket to the path
    print(f"Starting up on {socket_path}")
    sock.bind(socket_path)

    # Listen for incoming connections
    sock.listen(1)

    while True:
        print("Waiting for a connection")
        connection, client_address = sock.accept()
        try:
            while True:
                run()
        finally:
            # Clean up the connection
            connection.close()
