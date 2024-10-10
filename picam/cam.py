import socket
from picamera2 import Picamera2
import struct
import threading
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

            a_bytes = a.tobytes()
            qr.put(a_bytes)
    finally:
        # Clean up
        picam2.stop()


def run(w1, h1, w2, h2):
    get = sock.recv(4)
    if len(get) == 0:
        print("Connection closed, exiting...")
        cam1_send_q.put_nowait(False)
        cam2_send_q.put_nowait(False)
        return

    get = struct.unpack("!I", get)[0]
    if get != 1:
        return

    cam1_send_q.put_nowait(True)
    cam2_send_q.put_nowait(True)

    img1 = cam1_receive_q.get(timeout=2)
    img2 = cam2_receive_q.get(timeout=2)

    # TODO: send the actual dimension the camera selected
    sock.sendall(struct.pack("!I", w1))
    sock.sendall(struct.pack("!I", h1))
    sock.sendall(struct.pack("!I", len(img1)))
    sock.sendall(img1)

    _send = sock.recv(4)
    _send = struct.unpack("!I", _send)[0]
    if _send != 2:
        return

    sock.sendall(struct.pack("!I", w2))
    sock.sendall(struct.pack("!I", h2))
    sock.sendall(struct.pack("!I", len(img2)))
    sock.sendall(img2)


if __name__ == "__main__":
    import os
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("--w1", type=int)
    parser.add_argument("--h1", type=int)
    parser.add_argument("--w2", type=int)
    parser.add_argument("--h2", type=int)
    parser.add_argument("--socket")

    args = parser.parse_args()

    w1 = args.w1 if args.w1 else 1296
    h1 = args.h1 if args.h1 else 972

    w2 = args.w2 if args.w2 else 1296
    h2 = args.h2 if args.h2 else 972

    thread_cam0 = threading.Thread(
        target=capture_and_send, args=(0, cam1_send_q, cam1_receive_q, w1, h1)
    )
    thread_cam1 = threading.Thread(
        target=capture_and_send, args=(1, cam2_send_q, cam2_receive_q, w2, h2)
    )

    thread_cam0.start()
    thread_cam1.start()

    # Socket file path
    socket_path = args.socket if args.socket else "/tmp/picam.sock"

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
                run(w1, h1, w2, h2)
        finally:
            # Clean up the connection
            connection.close()
