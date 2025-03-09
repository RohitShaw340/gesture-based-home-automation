from picamera2 import Picamera2
import threading
import argparse
import time

config = {"process_id": "cam", "server_address": "/tmp/gesurease.sock"}
W_DEFAULT = 1296
H_DEFAULT = 972
OUT_DEFAULT = "."


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
    parser.add_argument("-w1", type=int, default=W_DEFAULT)
    parser.add_argument("-h1", type=int, default=H_DEFAULT)
    parser.add_argument("-w2", type=int, default=W_DEFAULT)
    parser.add_argument("-h2", type=int, default=H_DEFAULT)
    parser.add_argument("-f", "--file-name", type=str)
    parser.add_argument("-o", "--out-dir", type=str, default=OUT_DEFAULT)

    args = parser.parse_args()

    now = datetime.datetime.now()
    fn = args.file_name if args.file_name else f"{now}.jpeg"

    w1 = args.w1
    h1 = args.h1

    w2 = args.w2
    h2 = args.h2

    dir1 = f"{args.out_dir}/cam1"
    dir2 = f"{args.out_dir}/cam2"

    Path(dir1).mkdir(parents=True, exist_ok=True)
    Path(dir2).mkdir(parents=True, exist_ok=True)

    thread_cam0 = threading.Thread(
        target=capture_and_save, args=(0, w1, h1, f"{dir1}/{fn}")
    )
    thread_cam1 = threading.Thread(
        target=capture_and_save, args=(1, w2, h2, f"{dir2}/{fn}")
    )

    thread_cam0.start()
    thread_cam1.start()
