// automatically generated by the FlatBuffers compiler, do not modify

import { RandomBytes, RandomBytesT } from '../client-request/random-bytes.js';
import { UserInputResponse, UserInputResponseT } from '../client-request/user-input-response.js';
import { ApplicationMessage, ApplicationMessageT } from '../common/application-message.js';
import { GetSecretRequest, GetSecretRequestT } from '../common/get-secret-request.js';
import { GetSecretResponse, GetSecretResponseT } from '../common/get-secret-response.js';


export enum InboundDelegateMsgType {
  NONE = 0,
  common_ApplicationMessage = 1,
  common_GetSecretResponse = 2,
  RandomBytes = 3,
  UserInputResponse = 4,
  common_GetSecretRequest = 5
}

export function unionToInboundDelegateMsgType(
  type: InboundDelegateMsgType,
  accessor: (obj:ApplicationMessage|GetSecretRequest|GetSecretResponse|RandomBytes|UserInputResponse) => ApplicationMessage|GetSecretRequest|GetSecretResponse|RandomBytes|UserInputResponse|null
): ApplicationMessage|GetSecretRequest|GetSecretResponse|RandomBytes|UserInputResponse|null {
  switch(InboundDelegateMsgType[type]) {
    case 'NONE': return null; 
    case 'common_ApplicationMessage': return accessor(new ApplicationMessage())! as ApplicationMessage;
    case 'common_GetSecretResponse': return accessor(new GetSecretResponse())! as GetSecretResponse;
    case 'RandomBytes': return accessor(new RandomBytes())! as RandomBytes;
    case 'UserInputResponse': return accessor(new UserInputResponse())! as UserInputResponse;
    case 'common_GetSecretRequest': return accessor(new GetSecretRequest())! as GetSecretRequest;
    default: return null;
  }
}

export function unionListToInboundDelegateMsgType(
  type: InboundDelegateMsgType, 
  accessor: (index: number, obj:ApplicationMessage|GetSecretRequest|GetSecretResponse|RandomBytes|UserInputResponse) => ApplicationMessage|GetSecretRequest|GetSecretResponse|RandomBytes|UserInputResponse|null, 
  index: number
): ApplicationMessage|GetSecretRequest|GetSecretResponse|RandomBytes|UserInputResponse|null {
  switch(InboundDelegateMsgType[type]) {
    case 'NONE': return null; 
    case 'common_ApplicationMessage': return accessor(index, new ApplicationMessage())! as ApplicationMessage;
    case 'common_GetSecretResponse': return accessor(index, new GetSecretResponse())! as GetSecretResponse;
    case 'RandomBytes': return accessor(index, new RandomBytes())! as RandomBytes;
    case 'UserInputResponse': return accessor(index, new UserInputResponse())! as UserInputResponse;
    case 'common_GetSecretRequest': return accessor(index, new GetSecretRequest())! as GetSecretRequest;
    default: return null;
  }
}
