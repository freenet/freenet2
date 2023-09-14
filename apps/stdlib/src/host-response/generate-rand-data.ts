// automatically generated by the FlatBuffers compiler, do not modify

/* eslint-disable @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */

import * as flatbuffers from 'flatbuffers';



export class GenerateRandData implements flatbuffers.IUnpackableObject<GenerateRandDataT> {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):GenerateRandData {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsGenerateRandData(bb:flatbuffers.ByteBuffer, obj?:GenerateRandData):GenerateRandData {
  return (obj || new GenerateRandData()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsGenerateRandData(bb:flatbuffers.ByteBuffer, obj?:GenerateRandData):GenerateRandData {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new GenerateRandData()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

wrappedState(index: number):number|null {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.readUint8(this.bb!.__vector(this.bb_pos + offset) + index) : 0;
}

wrappedStateLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

wrappedStateArray():Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? new Uint8Array(this.bb!.bytes().buffer, this.bb!.bytes().byteOffset + this.bb!.__vector(this.bb_pos + offset), this.bb!.__vector_len(this.bb_pos + offset)) : null;
}

static startGenerateRandData(builder:flatbuffers.Builder) {
  builder.startObject(1);
}

static addWrappedState(builder:flatbuffers.Builder, wrappedStateOffset:flatbuffers.Offset) {
  builder.addFieldOffset(0, wrappedStateOffset, 0);
}

static createWrappedStateVector(builder:flatbuffers.Builder, data:number[]|Uint8Array):flatbuffers.Offset {
  builder.startVector(1, data.length, 1);
  for (let i = data.length - 1; i >= 0; i--) {
    builder.addInt8(data[i]!);
  }
  return builder.endVector();
}

static startWrappedStateVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(1, numElems, 1);
}

static endGenerateRandData(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  builder.requiredField(offset, 4) // wrapped_state
  return offset;
}

static createGenerateRandData(builder:flatbuffers.Builder, wrappedStateOffset:flatbuffers.Offset):flatbuffers.Offset {
  GenerateRandData.startGenerateRandData(builder);
  GenerateRandData.addWrappedState(builder, wrappedStateOffset);
  return GenerateRandData.endGenerateRandData(builder);
}

unpack(): GenerateRandDataT {
  return new GenerateRandDataT(
    this.bb!.createScalarList<number>(this.wrappedState.bind(this), this.wrappedStateLength())
  );
}


unpackTo(_o: GenerateRandDataT): void {
  _o.wrappedState = this.bb!.createScalarList<number>(this.wrappedState.bind(this), this.wrappedStateLength());
}
}

export class GenerateRandDataT implements flatbuffers.IGeneratedObject {
constructor(
  public wrappedState: (number)[] = []
){}


pack(builder:flatbuffers.Builder): flatbuffers.Offset {
  const wrappedState = GenerateRandData.createWrappedStateVector(builder, this.wrappedState);

  return GenerateRandData.createGenerateRandData(builder,
    wrappedState
  );
}
}
