CC      ?= cc
CFLAGS  ?= -Wall -Wextra -g
LDFLAGS ?=
LDLIBS  ?= -lcrypt

TARGET  := minimal-sudo
SRCS    := main.c auth.c
OBJS    := $(SRCS:.c=.o)

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(LDFLAGS) -o $@ $(OBJS) $(LDLIBS)

main.o: main.c auth.h
auth.o: auth.c auth.h

clean:
	rm -f $(OBJS) $(TARGET)
