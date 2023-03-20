import { TestBed } from '@angular/core/testing';

import { AutoTranslationService } from './auto-translation.service';

describe('AutoTranslationService', () => {
  let service: AutoTranslationService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(AutoTranslationService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
