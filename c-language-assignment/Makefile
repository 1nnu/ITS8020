CC      ?= cc
CFLAGS  ?= -Wall -Wextra -g
LDFLAGS ?=
LDLIBS  ?= -lcrypt

TARGET  := minimal-sudo
SRCS    := main.c auth.c run.c
OBJS    := $(SRCS:.c=.o)

.PHONY: all clean install

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(LDFLAGS) -o $@ $(OBJS) $(LDLIBS)

install: $(TARGET)
	sudo chown root:root $(TARGET)
	sudo chmod u+s $(TARGET)

main.o: main.c auth.h run.h
auth.o: auth.c auth.h
run.o: run.c run.h

clean:
	rm -f $(OBJS) $(TARGET)
