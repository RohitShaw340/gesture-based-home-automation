from picamera2 import Picamera2
import threading
import argparse
import time

config = {"process_id": "cam", "server_address": "/tmp/gesurease.sock"}


def capture_and_save(camera_id, w, h, file_name):
    picam2 = Picamera2(camera_num=camera_id)
    camera_config = picam2.create_still_configuration(
        main={
            "size": (w, h),
        },
        queue=False
    )  # Adjust settings as needed
    picam2.configure(camera_config)
    picam2.start()
    time.sleep(2)
    try:
        # Capture image
        picam2.capture_file(file_name)
    finally:
        # Clean up
        picam2.stop()


def run():
    pass


if __name__ == "__main__":
    import datetime
    from pathlib import Path

    parser = argparse.ArgumentParser()
    parser.add_argument("w1", type=int)
    parser.add_argument("h1", type=int)
    parser.add_argument("w2", type=int)
    parser.add_argument("h2", type=int)
    parser.add_argument("--f1")
    parser.add_argument("--f2")
    parser.add_argument("--file-format")

    args = parser.parse_args()

    now = datetime.datetime.now()
    file_format = args.file_format if args.file_format else "jpeg"

    w1 = args.w1
    h1 = args.h1
    f1 = args.f1 if args.f1 else f"cam1/{now}.{file_format}"

    w2 = args.w2
    h2 = args.h2
    f2 = args.f2 if args.f2 else f"cam2/{now}.{file_format}"

    Path("cam1").mkdir(parents=True, exist_ok=True)
    Path("cam2").mkdir(parents=True, exist_ok=True)

    thread_cam0 = threading.Thread(
        target=capture_and_save, args=(0, w1, h1, f1)
    )
    thread_cam1 = threading.Thread(
        target=capture_and_save, args=(1, w2, h2, f2)
    )

    thread_cam0.start()
    thread_cam1.start()
