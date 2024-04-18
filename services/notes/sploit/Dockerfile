FROM ubuntu:22.04

RUN apt update && apt install -y socat python3-pip
RUN pip install pwn

WORKDIR /app

COPY sploit.py libc.so.6 get_flags.py run_sploit.sh .

CMD ["sh", "/app/run_sploit.sh"]
