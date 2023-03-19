import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AutoTranslationComponent } from './auto-translation.component';

describe('AutoTranslationComponent', () => {
  let component: AutoTranslationComponent;
  let fixture: ComponentFixture<AutoTranslationComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ AutoTranslationComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(AutoTranslationComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
