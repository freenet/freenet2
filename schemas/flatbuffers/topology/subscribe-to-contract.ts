// automatically generated by the FlatBuffers compiler, do not modify

import * as flatbuffers from 'flatbuffers';

export class SubscribeToContract {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):SubscribeToContract {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsSubscribeToContract(bb:flatbuffers.ByteBuffer, obj?:SubscribeToContract):SubscribeToContract {
  return (obj || new SubscribeToContract()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsSubscribeToContract(bb:flatbuffers.ByteBuffer, obj?:SubscribeToContract):SubscribeToContract {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new SubscribeToContract()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

requester():string|null
requester(optionalEncoding:flatbuffers.Encoding):string|Uint8Array|null
requester(optionalEncoding?:any):string|Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.__string(this.bb_pos + offset, optionalEncoding) : null;
}

transaction():string|null
transaction(optionalEncoding:flatbuffers.Encoding):string|Uint8Array|null
transaction(optionalEncoding?:any):string|Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.__string(this.bb_pos + offset, optionalEncoding) : null;
}

key():string|null
key(optionalEncoding:flatbuffers.Encoding):string|Uint8Array|null
key(optionalEncoding?:any):string|Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? this.bb!.__string(this.bb_pos + offset, optionalEncoding) : null;
}

contractLocation():number {
  const offset = this.bb!.__offset(this.bb_pos, 10);
  return offset ? this.bb!.readFloat64(this.bb_pos + offset) : 0.0;
}

atPeer():string|null
atPeer(optionalEncoding:flatbuffers.Encoding):string|Uint8Array|null
atPeer(optionalEncoding?:any):string|Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 12);
  return offset ? this.bb!.__string(this.bb_pos + offset, optionalEncoding) : null;
}

atPeerLocation():number {
  const offset = this.bb!.__offset(this.bb_pos, 14);
  return offset ? this.bb!.readFloat64(this.bb_pos + offset) : 0.0;
}

static startSubscribeToContract(builder:flatbuffers.Builder) {
  builder.startObject(6);
}

static addRequester(builder:flatbuffers.Builder, requesterOffset:flatbuffers.Offset) {
  builder.addFieldOffset(0, requesterOffset, 0);
}

static addTransaction(builder:flatbuffers.Builder, transactionOffset:flatbuffers.Offset) {
  builder.addFieldOffset(1, transactionOffset, 0);
}

static addKey(builder:flatbuffers.Builder, keyOffset:flatbuffers.Offset) {
  builder.addFieldOffset(2, keyOffset, 0);
}

static addContractLocation(builder:flatbuffers.Builder, contractLocation:number) {
  builder.addFieldFloat64(3, contractLocation, 0.0);
}

static addAtPeer(builder:flatbuffers.Builder, atPeerOffset:flatbuffers.Offset) {
  builder.addFieldOffset(4, atPeerOffset, 0);
}

static addAtPeerLocation(builder:flatbuffers.Builder, atPeerLocation:number) {
  builder.addFieldFloat64(5, atPeerLocation, 0.0);
}

static endSubscribeToContract(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  builder.requiredField(offset, 4) // requester
  builder.requiredField(offset, 6) // transaction
  builder.requiredField(offset, 8) // key
  builder.requiredField(offset, 12) // at_peer
  return offset;
}

static createSubscribeToContract(builder:flatbuffers.Builder, requesterOffset:flatbuffers.Offset, transactionOffset:flatbuffers.Offset, keyOffset:flatbuffers.Offset, contractLocation:number, atPeerOffset:flatbuffers.Offset, atPeerLocation:number):flatbuffers.Offset {
  SubscribeToContract.startSubscribeToContract(builder);
  SubscribeToContract.addRequester(builder, requesterOffset);
  SubscribeToContract.addTransaction(builder, transactionOffset);
  SubscribeToContract.addKey(builder, keyOffset);
  SubscribeToContract.addContractLocation(builder, contractLocation);
  SubscribeToContract.addAtPeer(builder, atPeerOffset);
  SubscribeToContract.addAtPeerLocation(builder, atPeerLocation);
  return SubscribeToContract.endSubscribeToContract(builder);
}
}
