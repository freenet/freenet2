// automatically generated by the FlatBuffers compiler, do not modify

import * as flatbuffers from 'flatbuffers';

import { ContractRequestType, unionToContractRequestType, unionListToContractRequestType } from '../client-request/contract-request-type.js';
import { Get, GetT } from '../client-request/get.js';
import { Put, PutT } from '../client-request/put.js';
import { Subscribe, SubscribeT } from '../client-request/subscribe.js';
import { Update, UpdateT } from '../client-request/update.js';


export class ContractRequest implements flatbuffers.IUnpackableObject<ContractRequestT> {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):ContractRequest {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsContractRequest(bb:flatbuffers.ByteBuffer, obj?:ContractRequest):ContractRequest {
  return (obj || new ContractRequest()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsContractRequest(bb:flatbuffers.ByteBuffer, obj?:ContractRequest):ContractRequest {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new ContractRequest()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

contractRequestType():ContractRequestType {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.readUint8(this.bb_pos + offset) : ContractRequestType.NONE;
}

contractRequest<T extends flatbuffers.Table>(obj:any):any|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.__union(obj, this.bb_pos + offset) : null;
}

static startContractRequest(builder:flatbuffers.Builder) {
  builder.startObject(2);
}

static addContractRequestType(builder:flatbuffers.Builder, contractRequestType:ContractRequestType) {
  builder.addFieldInt8(0, contractRequestType, ContractRequestType.NONE);
}

static addContractRequest(builder:flatbuffers.Builder, contractRequestOffset:flatbuffers.Offset) {
  builder.addFieldOffset(1, contractRequestOffset, 0);
}

static endContractRequest(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  builder.requiredField(offset, 6) // contract_request
  return offset;
}

static createContractRequest(builder:flatbuffers.Builder, contractRequestType:ContractRequestType, contractRequestOffset:flatbuffers.Offset):flatbuffers.Offset {
  ContractRequest.startContractRequest(builder);
  ContractRequest.addContractRequestType(builder, contractRequestType);
  ContractRequest.addContractRequest(builder, contractRequestOffset);
  return ContractRequest.endContractRequest(builder);
}

unpack(): ContractRequestT {
  return new ContractRequestT(
    this.contractRequestType(),
    (() => {
      const temp = unionToContractRequestType(this.contractRequestType(), this.contractRequest.bind(this));
      if(temp === null) { return null; }
      return temp.unpack()
  })()
  );
}


unpackTo(_o: ContractRequestT): void {
  _o.contractRequestType = this.contractRequestType();
  _o.contractRequest = (() => {
      const temp = unionToContractRequestType(this.contractRequestType(), this.contractRequest.bind(this));
      if(temp === null) { return null; }
      return temp.unpack()
  })();
}
}

export class ContractRequestT implements flatbuffers.IGeneratedObject {
constructor(
  public contractRequestType: ContractRequestType = ContractRequestType.NONE,
  public contractRequest: GetT|PutT|SubscribeT|UpdateT|null = null
){}


pack(builder:flatbuffers.Builder): flatbuffers.Offset {
  const contractRequest = builder.createObjectOffset(this.contractRequest);

  return ContractRequest.createContractRequest(builder,
    this.contractRequestType,
    contractRequest
  );
}
}
