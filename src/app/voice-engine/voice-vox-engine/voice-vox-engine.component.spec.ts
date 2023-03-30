import { ComponentFixture, TestBed } from '@angular/core/testing';

import { VoiceVoxEngineComponent } from './voice-vox-engine.component';

describe('VoiceVoxEngineComponent', () => {
  let component: VoiceVoxEngineComponent;
  let fixture: ComponentFixture<VoiceVoxEngineComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ VoiceVoxEngineComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(VoiceVoxEngineComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
