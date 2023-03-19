import { ComponentFixture, TestBed } from '@angular/core/testing';

import { VoiceEngineComponent } from './voice-engine.component';

describe('VoiceEngineComponent', () => {
  let component: VoiceEngineComponent;
  let fixture: ComponentFixture<VoiceEngineComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ VoiceEngineComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(VoiceEngineComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
