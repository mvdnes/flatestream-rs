$(OUT_DIR)/libminiz.a: csrc/miniz.c
	$(CC) -c csrc/miniz.c -fPIC -o $(OUT_DIR)/miniz.o
	$(AR) r $(OUT_DIR)/libminiz.a $(OUT_DIR)/miniz.o
