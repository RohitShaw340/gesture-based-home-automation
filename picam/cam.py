import socket
from picamera2 import Picamera2
import numpy as np
import struct
import threading
import json
from queue import Queue

# Configure the socket
HOST = "localhost"  # Replace with your Rust application's IPC endpoint
PORT = 5555  # Replace with the port number your Rust application listens on

config = {"process_id": "cam", "server_address": "/tmp/gesurease.sock"}

cam1_send_q = Queue()
cam1_receive_q = Queue()
cam2_send_q = Queue()
cam2_receive_q = Queue()


def capture_and_send(camera_id, qs, qr, w, h):
    picam2 = Picamera2(camera_num=camera_id)
    camera_config = picam2.create_still_configuration(
        main={"size": (w, h)},
        queue=False
    )  # Adjust settings as needed
    picam2.configure(camera_config)
    picam2.start()
    try:
        while True:
            if qs.get() is False:
                break

            # Capture image
            a = picam2.capture_array("main")

            # Converting the array 'a' to bytes using a.tobytes() method and storing it in 'a_bytes'
            a_bytes = a.tobytes()
            qr.put(a_bytes)
    finally:
        # Clean up
        picam2.stop()


def run():
    get = sock.recv(4)
    if len(get) == 0:
        print("Connection closed, exiting...")
        cam1_send_q.put_nowait(False)
        cam2_send_q.put_nowait(False)
        exit(1)

    get = struct.unpack("!I", get)[0]
    if get != 1:
        return

    cam1_send_q.put_nowait(True)
    cam2_send_q.put_nowait(True)

    img1 = cam1_receive_q.get(timeout=2)
    img2 = cam2_receive_q.get(timeout=2)

    sock.sendall(struct.pack("!I", len(img1)))
    sock.sendall(img1)

    _send = sock.recv(4)
    _send = struct.unpack("!I", _send)[0]
    if _send != 2:
        return

    sock.sendall(struct.pack("!I", len(img2)))
    sock.sendall(img2)


if __name__ == "__main__":
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.connect(config["server_address"])
    sock.setblocking(True)

    sock.sendall(config["process_id"].encode())

    print("Starting Cams. Waiting for img dimensions...")

    w1 = struct.unpack("!I", sock.recv(4))[0]
    h1 = struct.unpack("!I", sock.recv(4))[0]

    w2 = struct.unpack("!I", sock.recv(4))[0]
    h2 = struct.unpack("!I", sock.recv(4))[0]

    print("Dimensions received.")
    print(w1, h1, w2, h2)
    print("Starting cams...")

    thread_cam0 = threading.Thread(
        target=capture_and_send, args=(0, cam1_send_q, cam1_receive_q, w1, h1)
    )
    thread_cam1 = threading.Thread(
        target=capture_and_send, args=(1, cam2_send_q, cam2_receive_q, w2, h2)
    )

    thread_cam0.start()
    thread_cam1.start()

    while True:
        run()
