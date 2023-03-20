import { TestBed } from '@angular/core/testing';

import { VoiceEngineService } from './voice-engine.service';

describe('VoiceEngineService', () => {
  let service: VoiceEngineService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(VoiceEngineService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
