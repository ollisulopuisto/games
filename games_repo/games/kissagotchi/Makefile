.PHONY: all setup build test lint clean

all: build

setup:
	npm install

build:
	mkdir -p dist
	cp index.html style.css app.js dist/

test:
	npm run test

lint:
	npm run lint

clean:
	rm -rf dist
	rm -rf node_modules
