// automatically generated by the FlatBuffers compiler, do not modify

import * as flatbuffers from 'flatbuffers';

import { InboundDelegateMsgType, unionToInboundDelegateMsgType, unionListToInboundDelegateMsgType } from '../client-request/inbound-delegate-msg-type.js';
import { RandomBytes, RandomBytesT } from '../client-request/random-bytes.js';
import { UserInputResponse, UserInputResponseT } from '../client-request/user-input-response.js';
import { ApplicationMessage, ApplicationMessageT } from '../common/application-message.js';
import { GetSecretRequest, GetSecretRequestT } from '../common/get-secret-request.js';
import { GetSecretResponse, GetSecretResponseT } from '../common/get-secret-response.js';


export class InboundDelegateMsg implements flatbuffers.IUnpackableObject<InboundDelegateMsgT> {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):InboundDelegateMsg {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsInboundDelegateMsg(bb:flatbuffers.ByteBuffer, obj?:InboundDelegateMsg):InboundDelegateMsg {
  return (obj || new InboundDelegateMsg()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsInboundDelegateMsg(bb:flatbuffers.ByteBuffer, obj?:InboundDelegateMsg):InboundDelegateMsg {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new InboundDelegateMsg()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

inboundType():InboundDelegateMsgType {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.readUint8(this.bb_pos + offset) : InboundDelegateMsgType.NONE;
}

inbound<T extends flatbuffers.Table>(obj:any):any|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.__union(obj, this.bb_pos + offset) : null;
}

static startInboundDelegateMsg(builder:flatbuffers.Builder) {
  builder.startObject(2);
}

static addInboundType(builder:flatbuffers.Builder, inboundType:InboundDelegateMsgType) {
  builder.addFieldInt8(0, inboundType, InboundDelegateMsgType.NONE);
}

static addInbound(builder:flatbuffers.Builder, inboundOffset:flatbuffers.Offset) {
  builder.addFieldOffset(1, inboundOffset, 0);
}

static endInboundDelegateMsg(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  builder.requiredField(offset, 6) // inbound
  return offset;
}

static createInboundDelegateMsg(builder:flatbuffers.Builder, inboundType:InboundDelegateMsgType, inboundOffset:flatbuffers.Offset):flatbuffers.Offset {
  InboundDelegateMsg.startInboundDelegateMsg(builder);
  InboundDelegateMsg.addInboundType(builder, inboundType);
  InboundDelegateMsg.addInbound(builder, inboundOffset);
  return InboundDelegateMsg.endInboundDelegateMsg(builder);
}

unpack(): InboundDelegateMsgT {
  return new InboundDelegateMsgT(
    this.inboundType(),
    (() => {
      const temp = unionToInboundDelegateMsgType(this.inboundType(), this.inbound.bind(this));
      if(temp === null) { return null; }
      return temp.unpack()
  })()
  );
}


unpackTo(_o: InboundDelegateMsgT): void {
  _o.inboundType = this.inboundType();
  _o.inbound = (() => {
      const temp = unionToInboundDelegateMsgType(this.inboundType(), this.inbound.bind(this));
      if(temp === null) { return null; }
      return temp.unpack()
  })();
}
}

export class InboundDelegateMsgT implements flatbuffers.IGeneratedObject {
constructor(
  public inboundType: InboundDelegateMsgType = InboundDelegateMsgType.NONE,
  public inbound: ApplicationMessageT|GetSecretRequestT|GetSecretResponseT|RandomBytesT|UserInputResponseT|null = null
){}


pack(builder:flatbuffers.Builder): flatbuffers.Offset {
  const inbound = builder.createObjectOffset(this.inbound);

  return InboundDelegateMsg.createInboundDelegateMsg(builder,
    this.inboundType,
    inbound
  );
}
}
