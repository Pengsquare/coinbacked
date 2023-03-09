run: build
	cd build
	python3 -m http.server --directory build --bind 127.0.0.1

build: copy render
	
render:
	python3 render.py > build/index.html

copy:
	mkdir -p build
	cp src/css/* build
	cp src/res/* build
	cp src/script/* build

clean:
	rm -rf build