// automatically generated by the FlatBuffers compiler, do not modify

/* eslint-disable @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */

import * as flatbuffers from 'flatbuffers';

import { ContractInstanceId, ContractInstanceIdT } from '../common/contract-instance-id.js';


export class RelatedStateAndDeltaUpdate implements flatbuffers.IUnpackableObject<RelatedStateAndDeltaUpdateT> {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):RelatedStateAndDeltaUpdate {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsRelatedStateAndDeltaUpdate(bb:flatbuffers.ByteBuffer, obj?:RelatedStateAndDeltaUpdate):RelatedStateAndDeltaUpdate {
  return (obj || new RelatedStateAndDeltaUpdate()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsRelatedStateAndDeltaUpdate(bb:flatbuffers.ByteBuffer, obj?:RelatedStateAndDeltaUpdate):RelatedStateAndDeltaUpdate {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new RelatedStateAndDeltaUpdate()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

relatedTo(obj?:ContractInstanceId):ContractInstanceId|null {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? (obj || new ContractInstanceId()).__init(this.bb!.__indirect(this.bb_pos + offset), this.bb!) : null;
}

state(index: number):number|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.readUint8(this.bb!.__vector(this.bb_pos + offset) + index) : 0;
}

stateLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

stateArray():Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? new Uint8Array(this.bb!.bytes().buffer, this.bb!.bytes().byteOffset + this.bb!.__vector(this.bb_pos + offset), this.bb!.__vector_len(this.bb_pos + offset)) : null;
}

delta(index: number):number|null {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? this.bb!.readUint8(this.bb!.__vector(this.bb_pos + offset) + index) : 0;
}

deltaLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

deltaArray():Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? new Uint8Array(this.bb!.bytes().buffer, this.bb!.bytes().byteOffset + this.bb!.__vector(this.bb_pos + offset), this.bb!.__vector_len(this.bb_pos + offset)) : null;
}

static startRelatedStateAndDeltaUpdate(builder:flatbuffers.Builder) {
  builder.startObject(3);
}

static addRelatedTo(builder:flatbuffers.Builder, relatedToOffset:flatbuffers.Offset) {
  builder.addFieldOffset(0, relatedToOffset, 0);
}

static addState(builder:flatbuffers.Builder, stateOffset:flatbuffers.Offset) {
  builder.addFieldOffset(1, stateOffset, 0);
}

static createStateVector(builder:flatbuffers.Builder, data:number[]|Uint8Array):flatbuffers.Offset {
  builder.startVector(1, data.length, 1);
  for (let i = data.length - 1; i >= 0; i--) {
    builder.addInt8(data[i]!);
  }
  return builder.endVector();
}

static startStateVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(1, numElems, 1);
}

static addDelta(builder:flatbuffers.Builder, deltaOffset:flatbuffers.Offset) {
  builder.addFieldOffset(2, deltaOffset, 0);
}

static createDeltaVector(builder:flatbuffers.Builder, data:number[]|Uint8Array):flatbuffers.Offset {
  builder.startVector(1, data.length, 1);
  for (let i = data.length - 1; i >= 0; i--) {
    builder.addInt8(data[i]!);
  }
  return builder.endVector();
}

static startDeltaVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(1, numElems, 1);
}

static endRelatedStateAndDeltaUpdate(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  builder.requiredField(offset, 4) // related_to
  builder.requiredField(offset, 6) // state
  builder.requiredField(offset, 8) // delta
  return offset;
}

static createRelatedStateAndDeltaUpdate(builder:flatbuffers.Builder, relatedToOffset:flatbuffers.Offset, stateOffset:flatbuffers.Offset, deltaOffset:flatbuffers.Offset):flatbuffers.Offset {
  RelatedStateAndDeltaUpdate.startRelatedStateAndDeltaUpdate(builder);
  RelatedStateAndDeltaUpdate.addRelatedTo(builder, relatedToOffset);
  RelatedStateAndDeltaUpdate.addState(builder, stateOffset);
  RelatedStateAndDeltaUpdate.addDelta(builder, deltaOffset);
  return RelatedStateAndDeltaUpdate.endRelatedStateAndDeltaUpdate(builder);
}

unpack(): RelatedStateAndDeltaUpdateT {
  return new RelatedStateAndDeltaUpdateT(
    (this.relatedTo() !== null ? this.relatedTo()!.unpack() : null),
    this.bb!.createScalarList<number>(this.state.bind(this), this.stateLength()),
    this.bb!.createScalarList<number>(this.delta.bind(this), this.deltaLength())
  );
}


unpackTo(_o: RelatedStateAndDeltaUpdateT): void {
  _o.relatedTo = (this.relatedTo() !== null ? this.relatedTo()!.unpack() : null);
  _o.state = this.bb!.createScalarList<number>(this.state.bind(this), this.stateLength());
  _o.delta = this.bb!.createScalarList<number>(this.delta.bind(this), this.deltaLength());
}
}

export class RelatedStateAndDeltaUpdateT implements flatbuffers.IGeneratedObject {
constructor(
  public relatedTo: ContractInstanceIdT|null = null,
  public state: (number)[] = [],
  public delta: (number)[] = []
){}


pack(builder:flatbuffers.Builder): flatbuffers.Offset {
  const relatedTo = (this.relatedTo !== null ? this.relatedTo!.pack(builder) : 0);
  const state = RelatedStateAndDeltaUpdate.createStateVector(builder, this.state);
  const delta = RelatedStateAndDeltaUpdate.createDeltaVector(builder, this.delta);

  return RelatedStateAndDeltaUpdate.createRelatedStateAndDeltaUpdate(builder,
    relatedTo,
    state,
    delta
  );
}
}
