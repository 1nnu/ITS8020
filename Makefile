# Source: https://dev.to/medunes/build-c-projects-like-a-pro-a-guide-to-idiomatic-makefiles-53b6
CC      ?= cc
CFLAGS  ?= -Wall -Wextra -g
LDFLAGS ?=
LDLIBS  ?= -lcrypt

TARGET  := minimal-sudo
SRCS    := main.c
OBJS    := $(SRCS:.c=.o)

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(LDFLAGS) -o $@ $(OBJS) $(LDLIBS)

clean:
	rm -f $(OBJS) $(TARGET)
