import { ComponentFixture, TestBed } from '@angular/core/testing';

import { VoiceVoxSpeakerComponent } from './voice-vox-speaker.component';

describe('VoiceVoxSpeakerComponent', () => {
  let component: VoiceVoxSpeakerComponent;
  let fixture: ComponentFixture<VoiceVoxSpeakerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ VoiceVoxSpeakerComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(VoiceVoxSpeakerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
