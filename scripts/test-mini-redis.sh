#!/bin/sh

mini-redis-cli get hello
mini-redis-cli set hello world
mini-redis-cli get hello
mini-redis-cli set hello xxxxx
mini-redis-cli get hello
