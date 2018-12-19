import { TestBed } from '@angular/core/testing';

import { HabdataService } from './habdata.service';

describe('HabdataService', () => {
  beforeEach(() => TestBed.configureTestingModule({}));

  it('should be created', () => {
    const service: HabdataService = TestBed.get(HabdataService);
    expect(service).toBeTruthy();
  });
});
