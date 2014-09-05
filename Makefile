$(OUT_DIR)/libminiz.a: csrc/miniz.c
	$(CC) -c csrc/miniz.c -fPIC -o /tmp/miniz.o
	$(AR) r $(OUT_DIR)/libminiz.a /tmp/miniz.o
